from logging.config import dictConfig
from flask import Flask, redirect
from flask import request
import stripeImp
import gallery

dictConfig({
    'version': 1,
    'formatters': {'default': {
        'format': '[%(asctime)s] %(levelname)s in %(module)s: %(message)s',
    }},
    'handlers': {'wsgi': {
        'class': 'logging.StreamHandler',
        'stream': 'ext://flask.logging.wsgi_errors_stream',
        'formatter': 'default'
    }},
    'root': {
        'level': 'DEBUG',
        'handlers': ['wsgi']
    }
})


app = Flask('gallerystore')
gallery.setLogger(app.logger)
stripeImp.setSalesLogger(app.logger)
@app.route('/ready', methods=['GET'])
def routeIsReady():
    if gallery.isReady(app):
        return '',203
    else:
        return '',500 

@app.route('/info',methods=['POST'])
def info():
    if not(request.is_json):
        return 'Expected content type JSON',400
    elif not(isinstance(request.json,list)):
        return 'Expected list of print IDs',400
    if any([ not(isinstance(x,str)) for x in request.json]):
        return 'Expected list of string print IDs',400
    return gallery.info(request.json)
    
@app.route('/checkout',methods=['POST'])
def checkout():
    supportedCartVersions=[0]
    if not(request.is_json):
        return 'Expected content type JSON',400
    elif not(isinstance(request.json,dict)):
        return 'Expected cart dictionary',400
    elif not('items' in request.json):
        return 'Expected items in cart',400
    elif not('version' in request.json):
        return 'Expected cart version',400
    elif not(request.json['version'] in supportedCartVersions):
        return 'Cart version not supported!',400
    elif len(request.json['items']) == 0 :
        return 'Expected at least one item in cart',400
    (val,code) = gallery.validateCart(request.json)
        
    if code != 200:
        return (val,code)
    else:
        return stripeImp.checkout(val)


if __name__== '__main__':
    app.run()