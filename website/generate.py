#!/usr/bin/env python3
import glob
import datetime
import hashlib
import re
import os
import json
import numpy as np
import base64
from jinja2 import Template
import jinja2.filters as filters
import datetime
import re
from PIL import Image
from PIL import ExifTags
from libxmp.utils import file_to_dict
import dropbox as db
import clone
from util import *

viewerPath = "view/{}.html"
pathTemplate = "img/thumbnails/{}_{}.jpg"



toLink =  lambda x: addSlash(StripHTMLExt(x))
filters.FILTERS['tolink'] = toLink





TAGS_NR  = {}
for k,v in ExifTags.TAGS.items():
    TAGS_NR[v] = k

def addMeta(dropbox,token):
    need_download = False
    metadata = {}
    name = os.path.splitext(dropbox['name'])[0]
    filename = "img/dropbox/{}.json".format(dropbox['id_stripped'])
    try:
        with open(filename,'r') as f:
            metadata=json.loads(f.read())
            if metadata['dropbox']['content_hash'] != dropbox['content_hash']:
                need_download=True
                os.remove("img/meta/{}.json".format(name))
    except FileNotFoundError:
        need_download=True
    
    if need_download:
        removePathNoFail('img/meta/{}.json'.format(name))
        db.downloadFile(dropbox,token)
        metadata['dropbox']=dropbox
        with open(filename,'w') as f:
            json.dump(metadata,f)
    return  (name,need_download)



def globFiles():
    ls = glob.glob('img/dropbox/*.json')
    if not ls:
        raise Exception('No image files found')
    else:
        return ls

def genThumbnails(id,img):
    sizes = [150,300,512,1024,3000]
    names = ['tiny', 'small','medium','large','huge']
    for (s,name) in zip(sizes,names):
        thumb = img.copy()
        thumb.thumbnail( (s,s) )
        path = pathTemplate.format(id,name)
        thumb.save(path,quality=85)
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
    for dropbox in files:
        with open(dropbox,'r') as f:
            dropbox=json.load(f)
            dropbox=dropbox['dropbox']
        f = 'img/raw/{}'.format(dropbox['name'])
        filename = dropbox['name']
        name = os.path.splitext(filename)[0]
        meta= "img/meta/{}.json".format(name)


        try:
            with open(meta,'r') as f:
                print("({}/{}) [skip]\r".format(i,numFiles),end='')
                obj= json.loads(f.read())
                yield obj

        except (FileNotFoundError,IOError):
            with Image.open(f) as img:
                exif = img._getexif()
                xmp = file_to_dict(f) 
                xmpUrl = "http://purl.org/dc/elements/1.1/"
                
                if xmpUrl in xmp: 
                    xmp=xmp["http://purl.org/dc/elements/1.1/"]
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
                
                w,h = img.size
                id = base64.urlsafe_b64encode(hashlib.sha1(name.encode('utf-8')).digest()).decode('utf-8')



                tag = lambda x : exif[TAGS_NR[x]] if TAGS_NR[x] in exif else None
                if purlTitle:
                    name=purlTitle
                    print("Using XMP title {}".format(name))
                avg=np.round(np.mean(np.array(img),axis=(0,1)))
                avghex= ('#%02x%02x%02x' % tuple(avg.astype(int)))
                obj= {
                    'name': name,
                    'id': id,
                    'dropbox': {
                        'id': dropbox['id'],
                        'rev': dropbox['content_hash']
                    },
                    'date': tag('DateTimeOriginal'),
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
               
                with open(meta,'w') as f:
                    json.dump(obj,f)
                yield obj
                print("({}/{}) [ ok ]\r".format(i,numFiles),end='')
        i=i+1
    print('')

def genInventory():
    dateKey = lambda x : datetime.datetime.strptime(x['date'], "%Y:%m:%d %H:%M:%S")
    inventory = sorted(list(processImages()),key=dateKey,reverse=True)
    return inventory

def genHTML():
    year =datetime.datetime.now().year
    inventory = genInventory()
    websiteName =os.getenv('WEBSITE_URL')
    if not websiteName:
        websiteName = "/"
    gAdId = os.getenv('G_ANALYTICS_ID')


    templates = glob.glob('templates/*.template')
    for t in templates:
        filename = os.path.basename(t)
        name = os.path.splitext(filename)[0]
        with open(t,'r') as tm:
            template= Template(tm.read())
            if name != "sitemap":
                hname = "{}.html".format(name)
            else:
                hname = "{}.xml".format(name)

            if name == "viewer":
                for (i,img) in zip(range(0,len(inventory)),inventory):
                    with open(img['view'],'w') as vf:
                        print("Generating view  ({}/{})\r".format(i+1,len(inventory)),end='')
                        prev = inventory[i-1]['view'] if i > 0 else None
                        next = inventory[i+1]['view'] if i+1 < len(inventory) else None
                        jsonPath= toJsonPath(img['view'])
                        vf.write(template.render(pic=img,inventory=inventory,index=i,prev=prev,next=next,year=year,json=toLink(jsonPath)))
                        with open(jsonPath ,'w') as jv:
                            toSrc = lambda img : "{} {}w".format(toLink(img['path']),img['width'])
                            obj = {
                                'name': img['name'],
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
                with open(hname,'w') as f :
                    print("Generating " + hname + "...")
                    if gAdId:
                        f.write(template.render(inventory=inventory,year=year,websiteName=websiteName,gAdId=gAdId))
                    else:
                        f.write(template.render(inventory=inventory,year=year,websiteName=websiteName))

    return inventory




def generateWebsite():
    clone.removePathNoFail('api/manifest.json')
    token =  os.getenv('DROPBOX_API_TOKEN')
    foundImgs = []
    newImgs=[]
    removedImgs = []
    for i  in db.getFileMeta(token):
       img,isNew= addMeta(i,token)
       foundImgs.append(img)
       if isNew:
           newImgs.append(img)

    for f in glob.glob('img/meta/*.json'):
        filename = os.path.basename(f)
        name = os.path.splitext(filename)[0]
        if name not in foundImgs:
            print('Purge metadata ' + name)
            meta ={}
            with open(f,'r') as f:
                meta = json.load(f)
                removedImgs.append(meta) 
            removeMeta(meta)

    for f in glob.glob('img/raw/*.jpg'):
        filename = os.path.basename(f)
        name = os.path.splitext(filename)[0]
        if  name not in foundImgs:
            print('Purge image ' + name)
            removePathNoFail(f)

    print('Generating website...')
    inventory=genHTML()
    with open('api/manifest.json','w') as f:
        json.dump({
            'last_update': datetime.datetime.now().isoformat(),
            'img': {
                'inventory': inventory,
                'new': newImgs,
                'removed': removedImgs
            }
        },f)

if not os.getenv('MASTER_NODE_URL'):
    generateWebsite()
else:
    clone.fetchWebsite(os.getenv('MASTER_NODE_URL'))
