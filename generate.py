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

viewerPath = "view/{}.html"
pathTemplate = "img/thumbnails/{}_{}.jpg"

def StripHTMLExt(link):
    if link is None :
        return link

    m=re.match('^(.*)\.html$',link)
    if m:
        return m.group(1)
    else:
        return link

def addSlash(link):
    if link is None:
        return link
    elif link[0] == '/':
        return link
    else:
        return '/' + link
def toJsonPath(link):
    if link is None:
        return link
    else:
        return StripHTMLExt(link)+'.json'

toLink =  lambda x: addSlash(StripHTMLExt(x))
filters.FILTERS['tolink'] = toLink



pictureNames = {}
try: 
    with open('img/names.json','r') as f:
        pictureNames = json.load(f)
except FileNotFoundError:
    pass


TAGS_NR  = {}
for k,v in ExifTags.TAGS.items():
    TAGS_NR[v] = k


def files():
    ls = glob.glob('img/raw/*.jpg')
    if not ls:
        raise Exception("No pictures found")
    else:
        return ls


def processImages(files=files()):
    numFiles = len(files)
    i = 1
    for f in files:
        filename = os.path.basename(f)
        name = os.path.splitext(filename)[0]
        meta= "img/meta/{}.json".format(name)
        try:
            with open(meta,'r') as f:
                print("({}/{}) Skipping {}.".format(i,numFiles,name))
                obj= json.loads(f.read())
                if obj['name'] in  pictureNames:
                    obj['name'] = pictureNames[obj['name']]
                yield obj

        except FileNotFoundError:
            with Image.open(f) as img:
                print("({}/{}) Processing {}.".format(i,numFiles,name))
                exif = img._getexif()
                thumbL = img.copy()
                thumbS = img.copy()
                thumbM = img.copy()
                thumbS.thumbnail( (512,512))
                thumbM.thumbnail( (1024,1024))
                thumbL.thumbnail( (3000,3000))
                id = base64.urlsafe_b64encode(hashlib.sha1(name.encode('utf-8')).digest()).decode('utf-8')
                spath =   pathTemplate.format(id,'small',progressive=True,quality=85)
                mpath =   pathTemplate.format(id,'medium',progressive=True,quality=85)
                lpath =   pathTemplate.format(id,'large',progressive=True,quality=85)



                tag = lambda x : exif[TAGS_NR[x]] if TAGS_NR[x] in exif else None
                thumbS.save(spath)
                thumbM.save(mpath)
                thumbL.save(lpath)

                #for k,v in exif.items():
                #    if k in ExifTags.TAGS:
                #        print("{}: {}".format(ExifTags.TAGS[k],v))
                if name in  pictureNames:
                    name = pictureNames[name]
                avg=np.round(np.mean(np.array(img),axis=(0,1)))
                avghex= ('#%02x%02x%02x' % tuple(avg.astype(int)))
                obj= {
                    'name': name,
                    'id': id,
                    'date': tag('DateTimeOriginal'),
                    'rating': tag('Rating'),
                    'view': viewerPath.format(id),
                    'Copyright': tag('Copyright'),
                    'colour': avghex,
                    'original': {
                        'path' : f,
                        'width': img.width,
                        'height': img.height
                    },
                    'thumbL': {
                        'path' : lpath,
                        'width': thumbL.width,
                        'height': thumbL.height
                    },
                    'thumbM': {
                        'path' : mpath,
                        'width': thumbM.width,
                        'height': thumbM.height
                    },
                    'thumbS': {
                        'path' : spath,
                        'width': thumbS.width,
                        'height': thumbS.height
                    }

                }

                with open(meta,'w') as f:
                    json.dump(obj,f)
                yield obj
        i=i+1

def genInventory():
    dateKey = lambda x : datetime.datetime.strptime(x['date'], "%Y:%m:%d %H:%M:%S")
    inventory = sorted(list(processImages()),key=dateKey,reverse=True)
    with open('img/inventory.json','w') as f:
        json.dump(inventory,f)
    return inventory

def genHTML():
    year =datetime.datetime.now().year
    inventory = genInventory()
    templates = glob.glob('templates/*.template')
    print('Generating html...')
    for t in templates:
        filename = os.path.basename(t)
        name = os.path.splitext(filename)[0]
        with open(t,'r') as tm:
            template= Template(tm.read())
            hname = "{}.html".format(name)
            if name == "viewer":
                for (i,img) in zip(range(0,len(inventory)),inventory):
                    with open(img['view'],'w') as vf:
                        print("Generating " + img['view'] + "...")
                        prev = inventory[i-1]['view'] if i > 0 else None
                        next = inventory[i+1]['view'] if i+1 < len(inventory) else None
                        jsonPath= toJsonPath(img['view'])
                        vf.write(template.render(pic=img,inventory=inventory,index=i,prev=prev,next=next,year=year,json=toLink(jsonPath)))
                        with open(jsonPath ,'w') as jv:
                            obj = {
                                'name': img['name'],
                                'id' : name,
                                'colour': img['colour'],
                                'path': toLink(img['thumbL']['path']),
                                'url':  toLink(img['view']),
                                'next': toLink(toJsonPath(next)),
                                'prev': toLink(toJsonPath(prev))
                            }
                            json.dump(obj,jv)

                            
            else:
                with open(hname,'w') as f :
                    print("Generating " + hname + "...")
                    f.write(template.render(inventory=inventory,year=year))
        
genHTML()
