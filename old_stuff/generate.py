#!/usr/bin/env python3
import glob
import datetime
import hashlib
import re
import os
import json
import numpy as np
import base64
from jinja2 import Template, Environment, FileSystemLoader
import jinja2.filters as filters
import datetime
import re
from PIL import Image,ImageOps
from PIL import ImageCms
from PIL import ExifTags
from libxmp import utils,XMPFiles,consts

import dropbox as db
import clone
import store
from util import *
from categories import sortbycategories,loadAndValidateCategories
import datetime

viewerPath = "view/{}.html"
pathTemplate = "img/thumbnails/{}_{}.jpg"

def hashId(name,id):
    return base64.urlsafe_b64encode(hashlib.sha1((name+id).encode('utf-8')).digest()).decode('utf-8')

toLink =  lambda x: addSlash(StripHTMLExt(x))


date2year = lambda x: datetime.datetime.strptime(x, "%Y:%m:%d %H:%M:%S").year
def sortByYears(inventory):
    newinventory = {}
    for pic in inventory:
        year = date2year(pic['date'])
        if not(year in newinventory):
            newinventory[year] = []
        newinventory[year].append(pic)
    return newinventory


filters.FILTERS['sortbyyears']      = sortByYears
filters.FILTERS['tolink']           = toLink
filters.FILTERS['date2year']        = date2year
filters.FILTERS['sortbycategories'] = sortbycategories


def addMeta(dropbox,token):
    need_download = False
    metadata = {}
    name = os.path.splitext(dropbox['name'])[0]
    hashedId = hashId(name,dropbox['id_stripped'])
    filename = "img/meta/{}.json".format(hashedId)
    try:
        with open(filename,'r') as f:
            metadata=json.loads(f.read())
            if metadata['dropbox']['rev'] != hashId(name,dropbox['content_hash']):
                need_download=True
    except FileNotFoundError:
        need_download=True
    
    if need_download:
        db.downloadFile(dropbox,token)
        metadata['dropbox'] = {
            'id': hashedId,
            'rev': hashId(name,dropbox['content_hash']),
            'outdated': True
        }
        metadata['name'] = name
        with open(filename,'w') as f:
            json.dump(metadata,f)
    return  metadata



def globFiles():
    ls = glob.glob('img/meta/*.json')
    if not ls:
        raise Exception('No image files found')
    else:
        return ls

def genThumbnails(id,img):
    sizes = [150,300,512,1024,3000,2048]
    names = ['tiny', 'small','medium','large','huge','print']
    icc_profile=img.info.get('icc_profile')
    for (s,name) in zip(sizes,names):
        thumb = img.copy()
        thumb.thumbnail( (s,s) )
        if name == 'print':
            thumb = ImageOps.expand(thumb,border=86,fill='white')
        path = pathTemplate.format(id,name)
        thumb.save(path,quality=85,optimize=True, icc_profile=icc_profile)
        yield (name, {
                        'path' : path,
                        'width': thumb.width,
                        'height': thumb.height
        })


def processImages():
    files = globFiles()
    numFiles = len(files)
    i = 1
    print('Processing images...')
    for meta in files:
        metafile=meta
        with open(meta,'r') as f:
            meta=json.load(f)


        if not meta['dropbox']['outdated']:
                print("({}/{}) [up to date]\r".format(i,numFiles),end='')
                yield meta

        else:
            meta['dropbox']['outdated']=False
            f = 'img/raw/{}.jpg'.format(meta['name'])
            try:
                with Image.open(f) as img:
                    exif = img._getexif()
                    xmpObj = XMPFiles(file_path=f, open_forupdate=False).get_xmp()
                    xmp =  utils.object_to_dict(xmpObj)

                    if consts.XMP_NS_DC in xmp: 
                        xmp=xmp[consts.XMP_NS_DC]
                        purlOrg={}
                        for k,v,_ in xmp:
                            purlOrg[k] = v
                        purlTitleKey =  "dc:title[1]"
                        purlTitle=None
                        if purlTitleKey in purlOrg:
                            purlTitle = purlOrg[purlTitleKey]
                            if purlTitle.strip() == "":
                                purlTitle=None
                    else:
                        purlOrg=None
                    displayname = meta['name'] 
                    if purlTitle:
                        displayname=purlTitle
                        print("Using XMP title {}".format(displayname))

                    w,h = img.size
                    tag = lambda x : exif[TAGS_NR[x]] if TAGS_NR[x] in exif else None
                    avg=np.round(np.mean(np.array(img),axis=(0,1)))
                    id = meta['dropbox']['id']
                    avghex= ('#%02x%02x%02x' % tuple(avg.astype(int)))
                    date = tag('DateTimeOriginal')
                    if not date:
                        date = tag('DateTimeDigitized')
                        if not date:
                            date = tag('DateTime')
                            if not date:
                                raise 'Image file contains no date information!'
                    obj= {
                        'name': meta['name'],
                        'displayname': displayname,
                        'dropbox': meta['dropbox'],
                        'date': date,
                        'xmp': xmpObj.serialize_to_str(),
                        'rating': tag('Rating'),
                        'view': viewerPath.format(id),
                        'Copyright': tag('Copyright'),
                        'colour': avghex,
                        'original': {
                            'path' : f,
                            'width': img.width,
                            'height': img.height
                        }
                    }
                    for (n,o) in genThumbnails(id,img):
                        obj[n]=o

                    with open(metafile,'w') as f:
                        json.dump(obj,f)
                    yield obj
                    print("({}/{}) [ ok ]\r".format(i,numFiles),end='')
            except FileNotFoundError as e:
                removeMeta(meta) # The meta data failed for some reason, 
                                 # Remove it to force reload
                raise e
        i=i+1
    print('')

def genInventory():
    dateKey = lambda x : datetime.datetime.strptime(x['date'], "%Y:%m:%d %H:%M:%S")
    inventory = sorted(list(processImages()),key=dateKey,reverse=True)
    return inventory

def genHTML():
    print('Generating website...')
    loadAndValidateCategories()
    
    year =datetime.datetime.now().year
    inventory = genInventory()
    storeData = store.generateStore(inventory)
    websiteName =os.getenv('WEBSITE_URL')

    if not websiteName:
        websiteName = "/"
    with open("version.json",'r') as f:
        versionObj = json.load(f)
        if versionObj['git']:
            gitSha = versionObj['git']
        else:
            gitSha = 'DEV'
    gAdId = os.getenv('G_ANALYTICS_ID')


    environment = Environment(loader=FileSystemLoader("templates/"))
    for templateName in environment.list_templates(".template"):
        template = environment.get_template(templateName)
        filename = os.path.basename(templateName)
        hname = os.path.splitext(filename)[0]
        name,suffix = os.path.splitext(hname)

         
        if suffix == ".html" or suffix == ".xml":
            if name == "viewer":
                for (i,img) in zip(range(0,len(inventory)),inventory):
                    print("Generating view  ({}/{})\r".format(i+1,len(inventory)),end='')
                    prev = inventory[i-1]['view'] if i > 0 else None
                    next = inventory[i+1]['view'] if i+1 < len(inventory) else None
                    jsonPath= toJsonPath(img['view'])
                    template.stream(pic=img,inventory=inventory,index=i,prev=prev,next=next,year=year,gitSha=gitSha,json=toLink(jsonPath)).dump(img['view'])
                    with open(jsonPath ,'w') as jv:
                        toSrc = lambda img : "{} {}w".format(toLink(img['path']),img['width'])
                        obj = {
                            'name': img['displayname'],
                            'id' : name,
                            'colour': img['colour'],
                            'path': toLink(img['large']['path']),
                            'url':  toLink(img['view']),
                            'srcset' : "{},{}".format(*list(map(lambda size : toSrc(img[size]),['large','huge']))),
                            'next': toLink(toJsonPath(next)),
                            'prev': toLink(toJsonPath(prev))
                        }
                        json.dump(obj,jv)
                print('')
            else:
                if name == 'store' and not(storeData):
                    continue
                print("Generating " + hname + "...")
                if gAdId:
                    template.stream(inventory=inventory,year=year,websiteName=websiteName,gAdId=gAdId,gitSha=gitSha,storeData=storeData).dump(hname)
                else:
                    template.stream(inventory=inventory,year=year,websiteName=websiteName,gitSha=gitSha,storeData=storeData).dump(hname)
        elif suffix == ".css":
            print("WARN: Ignoring {}".format(name))
    return inventory





def fetchDropbox():
    token =  os.getenv('DROPBOX_API_TOKEN')
    foundMeta = []
    newMeta=[]
    removedMeta = []
    removePathNoFail('api/manifest.json')
    removePathNoFail('api/sitedata.json')
    
    for i  in db.getFileMeta(token):
       if i['name'] == 'sitedata.json':
           db.downloadSitedata(i,token)
       else:
          meta= addMeta(i,token)
          foundMeta.append(meta)
          if meta['dropbox']['outdated']:
              newMeta.append(meta)
    

    for f in glob.glob('img/meta/*.json'):
        filename = os.path.basename(f)
        id = os.path.splitext(filename)[0]
        if id not in map(lambda x: x['dropbox']['id'],foundMeta):
            print('Purge metadata ' + filename)
            meta ={}
            with open(f,'r') as f:
                meta = json.load(f)
            removeMeta(meta)

    for f in glob.glob('img/raw/*.jpg'):
        filename = os.path.basename(f)
        name = os.path.splitext(filename)[0]
        if  name not in map(lambda x : x['name'],foundMeta) :
            print('Purge image ' + name)
            removePathNoFail(f)
    try: 
        with open('api/sitedata.json','r') as f:
            sitedata = json.load(f)
    except Exception as e:
        print("Failed do find sitedata.json in dropbox folder!")
        raise

    inventory=genHTML()
    with open('api/manifest.json','w') as f:
        json.dump({
            'last_update': datetime.datetime.now().isoformat(),
            'host': os.getenv('HOSTNAME'),
            'version': 5,
            'img': {
                'inventory': inventory,
                'new': newMeta,
                'removed': removedMeta
            }
        },f)



def main():
    try:    

        if not os.getenv('MASTER_NODE_URL'):
            fetchDropbox()
        else:
            try:
                dropboxAvailable=False
                if os.getenv('DROPBOX_API_TOKEN'):
                    dropboxAvailable=True
                manifest=clone.fetchWebsite(os.getenv('MASTER_NODE_URL'),dropboxAvailable)
            except Exception as e:
                if dropboxAvailable:
                    print('Tried to clone master website but failed, reverting to dropbox')
                    fetchDropbox()
                    return
                else:
                    raise e
            genHTML()
            with open('api/manifest.json','w') as f:
                json.dump(manifest,f)

    except Exception as e:
        clone.removePathNoFail('api/manifest.json')
        raise e
main()