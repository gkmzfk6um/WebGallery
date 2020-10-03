import requests
from requests.adapters import HTTPAdapter
from requests.packages.urllib3.util.retry import Retry
import os
import json
import datetime
import shutil
from util import *


adapter = HTTPAdapter(max_retries=Retry(total=10, backoff_factor=1,status_forcelist=[404,403,429,500, 502, 503, 504] ) )
httpInitial = requests.Session()
httpInitial.mount("https://", adapter)
httpInitial.mount("http://", adapter)

adapter2 = HTTPAdapter(max_retries=Retry(total=5, backoff_factor=0.5,status_forcelist=[403,429,500, 502, 503, 504] ) )
http = requests.Session()
http.mount("https://", adapter2)
http.mount("http://", adapter2)

def __get__manifest(url):
    fetchUrl = '{}/api/manifest.json'.format(url)
    r = httpInitial.get(fetchUrl)
    if r.ok:
        return r.json()
    else:
        raise Exception('Failed to get master manifest ({})'.format(url))

def diffManifest(client,master):
    new = {
        'last_update': datetime.datetime.now().isoformat(),
         'img': {
            'inventory':[],
            'all':[],
            'new': [],
            'removed': []
        } 
    } 
    print('Updating from manifest {} to {}'.format(client['last_update'],master['last_update']))
    for imgMaster in master['img']['inventory']:
        new['img']['inventory'].append(imgMaster)
        found = False
        for imgClient in client['img']['inventory']:
            if imgClient['dropbox']['id'] == imgMaster['dropbox']['id']:
                if imgClient['dropbox']['rev'] != imgMaster['dropbox']['rev']:
                    new['img']['new'].append(imgMaster)
                    new['img']['removed'].append(imgClient)
                found = True
                break
        if not found: 
            new['img']['new'].append(imgMaster)
    
    for imgClient in client['img']['inventory']:
        found = False
        for imgMaster in master['img']['inventory']:
            if imgClient['dropbox']['id'] == imgMaster['dropbox']['id']:
                found = True
                break
        if not found:
            new['img']['removed'].append(imgClient)

    print('Total {}, New {}, Removed {} '.format( len(new['img']['inventory']),len(new['img']['new']),len(new['img']['removed']) ))
    return new


def downloadFile(url,file):
    fileUrl = '{}/{}'.format(url,file)
    r = http.get(fileUrl,stream=True)
    if r.ok:
        r.raw.decode_content=True
        with open(file,'wb') as f:
            shutil.copyfileobj(r.raw,f)
        print("Downloaded {} ({})".format(file,sizeof_fmt(os.path.getsize(file))))
    else:
        print("Failed to download {} from master node".format(file))
        print(r)
        print(r.text)
        print(r.json())
        raise Exception('Failed to download file')

def cloneMeta(url,meta):
    downloadFile(url,meta['view'])
    downloadFile(url,toJsonPath(meta['view']))
    downloadFile(url,meta['tiny']['path'])
    downloadFile(url,meta['small']['path'])
    downloadFile(url,meta['medium']['path'])
    downloadFile(url,meta['large']['path'])
    downloadFile(url,meta['huge']['path'])


def fetchWebsite(url):
    print('Cloning {}'.format(url))
    url = 'http://{}'.format(url)
    master_manifest = __get__manifest(url)
    client_manifest = {
        'last_update':'1970-01-01T00:00:01.000000',
         'img': {
            'inventory':[],
            'all':[],
            'new': [],
            'removed': []
        } 
    } 
    try:
        with open('api/manifest.json','r') as f:
            tmp=json.load(f)
            client_manifest=tmp
    except Exception:
        print('Slave node is clean, cloning everything')
    
    new_manifest = diffManifest(client_manifest,master_manifest)
    for meta in new_manifest['img']['removed']:
        removeMeta(meta) 

    for meta in new_manifest['img']['new']:
        cloneMeta(url,meta)
    
    if len(new_manifest['img']['new']) > 0:
        downloadFile(url,'index.html')

    with open('api/manifest.json','w') as f:
        json.dump(new_manifest,f)
    
