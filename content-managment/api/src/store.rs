use serde::{Serialize,Deserialize};
use serde_json::json;
use crate::fetch::State;
use content_managment_datamodel::api::CheckoutCart;
use crate::util;
use log::{info,warn};
use content_managment_datamodel::datamodel::{ThumbnailSize,ImageMetadata};
use actix_web::http::header::ContentType;



lazy_static! {
    pub static ref API_KEY : String = std::env::var("STRIPE_API_TOKEN").unwrap();
    pub static ref GALLERY_URL :String = std::env::var("GALLERY_URL").unwrap();
}

#[derive(Serialize,Debug)]
pub struct StripeProductData
{
    name: String,
    description: String,
    images: Vec<String>
}

#[derive(Serialize,Debug)]
pub struct StripePriceData
{
    currency: String,
    product_data: StripeProductData,
    unit_amount: i32
}

#[derive(Serialize,Debug)]
pub struct StripeLineItem 
{
    price_data: StripePriceData,
    quantity: u8,
}

type StripeCart = Vec<StripeLineItem>;


pub fn validate_cart(state : &State,cart : CheckoutCart) -> Option<StripeCart>
{
    let mut stripe_cart = StripeCart::new();

    for (identifier,v) in cart.items
    {
        match util::find_print(&v.id, state.prints())
        {
             Some(print) => {
                let variant_name = &v.variant.name;
                if !print.variants.iter().any(|variant| variant==variant_name)
                {
                    warn!("Checkout failed, Version {} not found", variant_name);
                    return None;
                }

                let variant = state.prints().variants.get(variant_name).unwrap();
                if !(v.variant.height == variant.height 
                && v.variant.width == variant.width)
                {
                    warn!("Checkout failed, Variant {:#?} does not match {}", v.variant, variant_name);
                    return None;
                }

                let image = state.resources().resources.get(&v.id);
                if image.is_none()
                {
                    warn!("Checkout failed, Could not find image with id {}",v.id);
                    return None;
                }

                let sign_option = match v.variant.signature 
                {
                    0 => "Signature front",
                    1 => "Signature rear",
                    2 => "No signature!",
                    _ => return None
                };

                if format!("{}{}h{}w{}s", v.id,variant.height,variant.width,v.variant.signature) != identifier
                {
                    warn!("Checkout failed, Cart id's are note matching. Calculated {} received {}",identifier,identifier );
                    return None
                }

                let image = image.unwrap().as_data::<ImageMetadata>();
                let large_thumbnail = state.resources().resources.get(image.variants.get(&ThumbnailSize::Large).unwrap()).unwrap();

                
                stripe_cart.push(
                    StripeLineItem {
                        price_data: StripePriceData 
                        {
                            currency: variant.price.cur.clone() ,
                            product_data: StripeProductData 
                            {
                                name: image.name.clone(),
                                description: format!("{}cm x {}cm - {}",variant.width, variant.height, sign_option),
                                images: vec![ format!("{}/{}",*GALLERY_URL,large_thumbnail.path().display().to_string().replace(" ","%20") ) ]
                            },
                            unit_amount: (variant.price.value * 100) as i32
                        },
                        quantity: v.quantity
                    }
                )



             }
             None => {
                warn!("CheckoutFailed, Print {} not found",&v.id);
                return None; 
            }
        }
        

    }

    return Some(stripe_cart);
}

#[derive(Deserialize)]
pub struct StripeSession 
{
    id: String,
    object: String,
    url: String
}

pub async fn checkout_cart(cart: &StripeCart) -> actix_web::HttpResponse
{
    let allowed_countries= vec![
        // -- EU --------------------
        "AT","BE","BG","HR","CY","CZ",
        "DK","EE","FI","FR","DE","GR",
        "HU","IE","IT","LV","LT","LU",
        "MT","NL","PL","PT","RO","SK",
        "SI","ES","SE",
        // -- EUROPE ------------------
        "GB","NO",
        // -- North America -----------
        "CA","US"
    ];

    let shipping_cost : u32 = 0;
    let shipping_time : (u32,u32 )= (1,7);

    let payload = json!({
        "line_items":cart,
        "mode": "payment",
        "allow_promotion_codes": true,
        "shipping_address_collection" : { 
            "allowed_countries" : allowed_countries
        },
        "shipping_options": [
            {
                "shipping_rate_data": {
                    "type": "fixed_amount",
                    "fixed_amount": {
                        "amount": shipping_cost,
                        "currency": "Sek"
                    },
                    "display_name": "PostNord",
                    "delivery_estimate": {
                        "minimum": {
                            "unit": "business_day",
                            "value": shipping_time.0
                        },
                        "maximum": {
                            "unit": "business_day",
                            "value": shipping_time.1
                        }
                    }
                }
            }
        ],
        "phone_number_collection": {
            "enabled":true
        },
        "success_url": format!("{}/store/success",GALLERY_URL.to_string()),
        "cancel_url":  format!("{}/store/cancel",GALLERY_URL.to_string())
    });
    let payload = serde_urlencode_deep::ser::to_string(&payload).unwrap();


    
    info!("Initate stripe checkout {:#?}",cart);
    info!("{}", payload);
    
    let client = reqwest::Client::new();
    let resp = client.post("https://api.stripe.com/v1/checkout/sessions")
        .header(reqwest::header::AUTHORIZATION, format!("Bearer {}",*API_KEY))
        .header(reqwest::header::CONTENT_TYPE,reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded"))
        .body(payload)
        .send()
        .await;
    
    match resp 
    {
        Err(e) => {
            warn!("Could not send stripe checkout request! {:#?}",e);
            actix_web::HttpResponse::InternalServerError().body("Checkout failed")
        },
        Ok(stripe_checkout) => {
            if !stripe_checkout.status().is_success()
            {
                warn!("Could not send stripe checkout request! {:#?}",stripe_checkout);
                if let Ok(text) = stripe_checkout.text().await 
                {
                    warn!("Error response {}",text);

                }
                actix_web::HttpResponse::InternalServerError().body("Checkout failed")
            }
            else 
            {
                match  stripe_checkout.json::<StripeSession>().await 
                {
                    Err(e) => {
                        warn!("Failed to parse stripe api answer {:#?}",e);
                        actix_web::HttpResponse::InternalServerError().body("Checkout failed")
                    }
                    Ok(stripe_session) => {
                        if stripe_session.object == "checkout.session"
                        {
                            info!("Stripe checkout session {}", stripe_session.id);
                            actix_web::HttpResponse::Ok().insert_header(ContentType::plaintext()).body(stripe_session.url)
                        }
                        else 
                        {
                            warn!("Stripe did not return checkout.session buth rather {}",stripe_session.object);
                            actix_web::HttpResponse::InternalServerError().body("Checkout failed")
                        }
                    }
                }

            }
        }
    }

}