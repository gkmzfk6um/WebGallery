#!/usr/bin/env python3
import glob
import re
import os
import json
from jinja2 import Template
import datetime
from PIL import Image
from PIL import ExifTags

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
    pathTemplate = "img/thumbnails/{}_{}.jpg"
    for f in files:
        filename = os.path.basename(f)
        name = os.path.splitext(filename)[0]
        meta= "img/meta/{}.json".format(name)
        try:
            with open(meta,'r') as f:
                print("({}/{}) Skipping {}.".format(i,numFiles,name))
                yield json.loads(f.read())
        except FileNotFoundError:
            with Image.open(f) as img:
                print("({}/{}) Processing {}.".format(i,numFiles,name))
                exif = img._getexif()
                thumbL = img.copy()
                thumbS = img.copy()
                thumbS.thumbnail( (512,512))
                thumbL.thumbnail( (3000,3000))
                spath =   pathTemplate.format(name,'small',progressive=True,quality=75)
                lpath =   pathTemplate.format(name,'large',progressive=True,quality=75)



                tag = lambda x : exif[TAGS_NR[x]] if TAGS_NR[x] in exif else None
                thumbS.save(spath)
                thumbL.save(lpath)

                #for k,v in exif.items():
                #    if k in ExifTags.TAGS:
                #        print("{}: {}".format(ExifTags.TAGS[k],v))

                obj= {
                    'name': name,
                    'date': tag('DateTimeOriginal'),
                    'rating': tag('Rating'),
                    'Copyright': tag('Copyright'),
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
    inventory = genInventory()
    templates = glob.glob('templates/*.template')
    print('Generating html...')
    for t in templates:
        filename = os.path.basename(t)
        name = os.path.splitext(filename)[0]
        with open(t,'r') as tm:
            template= Template(tm.read())
            hname = "{}.html".format(name)
            with open(hname,'w') as f :
                print("Generating " + hname + "...")
                f.write(template.render(inventory=inventory))
        
genHTML()
