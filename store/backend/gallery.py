import os
from sre_constants import SUCCESS
import requests
import re
from requests.adapters import HTTPAdapter
from requests.packages.urllib3.util.retry import Retry


adapter = HTTPAdapter(max_retries=Retry(total=5, backoff_factor=0.5,status_forcelist=[403,429,500, 502, 503, 504] ) )
http = requests.Session()
http.mount("https://", adapter)
http.mount("http://", adapter)

galleryUrl = os.getenv("NODE_URL")
idRe = re.compile('[A-Za-z0-9-_=]+')
pathRe = re.compile('/([a-zA-z]+)((/([a-zA-z]+))*)')

signatureOptions = ['Sign front','Sign back', "Don't sign"]

def setLogger(newLogger):
    global logger
    logger = newLogger

def validateId(id):
    return re.match(idRe,id)

def validateDimension(dimension):
    return dimension > 10 and dimension < 100
def validateQuantity(quantity):
    return quantity > 0 and quantity < 100

def validateSignature(signature):
    return signature >= 0 and signature < len(signatureOptions) 


def copyPath(obj,newObj,path,nodeType,validate):
    match = re.match(pathRe,path)
    if not match:
        logger.debug('Match failed')
        logger.debug(match)
        logger.debug(path)
    root = match[1]
    if match[2]:
        if not (root in newObj):
            newObj[root] = {}
        newObj[root] = copyPath(obj[root],newObj[root],match[2],nodeType,validate)
        return newObj
    else:
        value = obj[root]
        if type(value) is nodeType:
            if validate(value):
                newObj[root] = value
                return newObj
            else:
                logger.debug(obj)
                logger.debug(path)
                logger.debug('Validation failed of key {}'.format(root))
        else:
            logger.debug(obj)
            logger.debug(path)
            logger.debug("Type {} isn't expected {}".format(type(value),nodeType))
    return None    

def cloneAndValidateDict(obj,paths):
    newObj = {}
    for (path,nodeType,validateCb) in paths:
        newObjTmp = copyPath(obj,newObj,path,nodeType,validateCb)
        if not(newObjTmp):
            return None
        else:
            newObj = newObjTmp
    return newObj

def validateCartItem(cartItem):
    paths = [
        ('/id',str,validateId),
        ('/variant/height',int,validateDimension),
        ('/variant/width',int,validateDimension),
        ('/variant/signature',int,validateSignature),
        ('/quantity',int,validateQuantity)
    ]
    return cloneAndValidateDict(cartItem,paths)

def buildCartIdString(item):
    logger.debug(item)
    return "{}{}h{}w{}s".format(item['id'],item['variant']['height'],item['variant']['width'],item['variant']['signature'])

supported = 4
def isReady(app):
    fetchUrl = '{}/api/manifest.json'.format(galleryUrl)
    r = http.get(fetchUrl)
    if r.ok:
        obj = r.json()
        if not 'version' in obj:
            app.logger.warn("Gallery API didn't provided version, assuming incompatibility")
            return False
        elif obj['version'] < supported:
            app.logger.warn("Gallery API incompatible. API v{} < supported v{}".format(obj['version'],supported))
            return False
    else:
        app.logger.warn('Failled to connect to Gallery')
        return False
    return True

def info(ids):
    if any( [ not(validateId(x)) for x in ids ]):
        return 'Invalid id',400
    
    fetchPrefix = '{}/api/print/'.format(galleryUrl)
    response = {
        'success': {},
        'failed': []
    }
    for id in ids:
        idFound = False
        url = fetchPrefix+id
        r= requests.get(fetchPrefix+id,allow_redirects=False,timeout=0.001)
        if r.ok:
            obj = r.json()
            assert id == obj['data']['dropbox']['id']
            response['success'][id] = {
                'name': obj['data']['displayname'],
                'variants': []
            }
            for variant in obj['variants']:
                response['success'][id]['variants'].append({
                    'width': variant['width'],
                    'height': variant['height'],
                    'price' : variant['price']['value']
                })
            idFound = True
                
        if not(idFound):
            response['failed'].append(id)
    return response

def crossReferenceCart(cart):
    idSet = set()
    for (k,v) in cart.items():
        idSet.add(v['id'])
    idInfo = info(list(idSet))
    if len(idInfo['failed']) > 0:
        return 'Id lookup failed',400
    idInfo = idInfo['success']

    for cartId,cartItem in cart.items():
        cartItemVariant = cartItem['variant']
        itemInfo = idInfo[cartItem['id']]
        variants = [variant for variant in itemInfo['variants'] if (variant['height'] == cartItemVariant['height'] and variant['width'] == cartItemVariant['width'] )]
        if len(variants) != 1:
            return 'Variant lookup failed',400
        logger.debug(itemInfo)
        cartItem['name'] = itemInfo['name']
        cartItem['price']= variants[0]['price']
        cartItem['signatureDescription'] = signatureOptions[cartItemVariant['signature']]
        cartItem['sizeDescription']      = "{} cm x {} cm".format(cartItemVariant['width'],cartItemVariant['height'])
    return cart,200


def validateCart(cart):
    validatedCart = {}
    for key,item in cart['items'].items():
        validatedItem = validateCartItem(item)
        if validatedItem is None:
            return 'Could not validate item',400
        idString = buildCartIdString(validatedItem)
        assert key == idString
        validatedCart[idString] = validatedItem
    return crossReferenceCart(validatedCart)