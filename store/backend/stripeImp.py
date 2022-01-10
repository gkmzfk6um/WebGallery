import logging
import stripe
import os
from flask import redirect


stripe.api_key = os.getenv('STRIPE_API_TOKEN')
galleryUrl = os.getenv('GALLERY_URL')
successUrl = "{}/store/success".format(galleryUrl)
cancelUrl = "{}/store/cancel".format(galleryUrl)
allowedCountires = [
    ## -- EU --------------------
    "AT","BE","BG","HR","CY","CZ",
    "DK","EE","FI","FR","DE","GR",
    "HU","IE","IT","LV","LT","LU",
    "MT","NL","PL","PT","RO","SK",
    "SI","ES","SE",
    # -- EUROPE ------------------
    "GB","NO",
    # -- North America -----------
    "CA","US"
]


def setSalesLogger(newLogger):
    global logger 
    logger = newLogger

def cart2lineItems(cart):
    lineItems = []
    for k,item in cart.items():
        description = "{}, {}".format(item['sizeDescription'],item['signatureDescription'])
        image = '{}/img/thumbnails/{}_large.jpg'.format(galleryUrl,item['id'] )
        lineItems.append(
        {
            'price_data': {
                'currency': 'sek',
                'product_data': {
                    'name': item['name'],
                    'description': description,
                    'images': [image]
                },
                'unit_amount': item['price']*100,
            },
            'quantity': item['quantity'],
        })
    return lineItems

def checkout(cart):
    logger.info("User checkout session initiated")
    logger.info(cart)
    session = stripe.checkout.Session.create(
        line_items=cart2lineItems(cart),
        mode='payment',
        allow_promotion_codes=True,
        shipping_address_collection= { 
            'allowed_countries' : allowedCountires
        },
        phone_number_collection= {
            'enabled':True
        },
        success_url=successUrl,
        cancel_url=cancelUrl
    )
    return session.url,200,{'Content-Type': 'text/text; charset=utf-8'} 