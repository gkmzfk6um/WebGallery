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

def hashId(name,id):
    return base64.urlsafe_b64encode(hashlib.sha1((name+id).encode('utf-8')).digest()).decode('utf-8')

toLink =  lambda x: addSlash(StripHTMLExt(x))
filters.FILTERS['tolink'] = toLink
TAGS_NR  = {}
for k,v in ExifTags.TAGS.items():
    TAGS_NR[v] = k

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
                displayname = meta['name'] 
                if purlTitle:
                    displayname=purlTitle
                    print("Using XMP title {}".format(displayname))
                
                w,h = img.size
                tag = lambda x : exif[TAGS_NR[x]] if TAGS_NR[x] in exif else None
                avg=np.round(np.mean(np.array(img),axis=(0,1)))
                id = meta['dropbox']['id']
                avghex= ('#%02x%02x%02x' % tuple(avg.astype(int)))
                obj= {
                    'name': meta['name'],
                    'displayname': displayname,
                    'dropbox': meta['dropbox'],
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
               
                with open(metafile,'w') as f:
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
    print('Generating website...')
    year =datetime.datetime.now().year
    inventory = genInventory()
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


    templates = glob.glob('templates/*.template')
    for t in templates:
        filename = os.path.basename(t)
        hname = os.path.splitext(filename)[0]
        name,suffix = os.path.splitext(hname)
        with open(t,'r') as tm:
            template= Template(tm.read())
            if suffix == ".html":
                if name == "viewer":
                    for (i,img) in zip(range(0,len(inventory)),inventory):
                        with open(img['view'],'w') as vf:
                            print("Generating view  ({}/{})\r".format(i+1,len(inventory)),end='')
                            prev = inventory[i-1]['view'] if i > 0 else None
                            next = inventory[i+1]['view'] if i+1 < len(inventory) else None
                            jsonPath= toJsonPath(img['view'])
                            vf.write(template.render(pic=img,inventory=inventory,index=i,prev=prev,next=next,year=year,gitSha=gitSha,json=toLink(jsonPath)))
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
                    with open(hname,'w') as f :
                        print("Generating " + hname + "...")
                        if gAdId:
                            f.write(template.render(inventory=inventory,year=year,websiteName=websiteName,gAdId=gAdId,gitSha=gitSha))
                        else:
                            f.write(template.render(inventory=inventory,year=year,websiteName=websiteName,gitSha=gitSha))
            elif suffix == ".css":
                filename = "css/{}-{}.css".format(name,gitSha)
                with open(filename,'w') as f:
                    print("Generating " + filename + "...")
                    f.write(template.render(inventory=inventory,year=year,gitSha=gitSha))

    return inventory





def fetchDropbox():
    token =  os.getenv('DROPBOX_API_TOKEN')
    foundMeta = []
    newMeta=[]
    removedMeta = []
    removePathNoFail('api/manifest.json')
    for i  in db.getFileMeta(token):
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

    inventory=genHTML()
    with open('api/manifest.json','w') as f:
        json.dump({
            'last_update': datetime.datetime.now().isoformat(),
            'host': os.getenv('HOSTNAME'),
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