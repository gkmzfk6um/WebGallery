import requests
from requests.adapters import HTTPAdapter
from requests.packages.urllib3.util.retry import Retry
import os
import json
import datetime
import shutil
from util import *

# 1 - Introduced embedded ICC profiles, needs cache rebuild
supportedVersion = 1


adapter = HTTPAdapter(max_retries=Retry(total=10, backoff_factor=1,status_forcelist=[404,403,429,500, 502, 503, 504] ) )
httpInitial = requests.Session()
httpInitial.mount("https://", adapter)
httpInitial.mount("http://", adapter)

adapter2 = HTTPAdapter(max_retries=Retry(total=5, backoff_factor=0.5,status_forcelist=[403,429,500, 502, 503, 504] ) )
http = requests.Session()
http.mount("https://", adapter2)
http.mount("http://", adapter2)

def __get__manifest(url,fastTimeout):
    fetchUrl = '{}/api/manifest.json'.format(url)
    if fastTimeout:
        r = http.get(fetchUrl)
    else:
        r = httpInitial.get(fetchUrl)
    if r.ok:
        obj = r.json()
        #Combability patch with older clients
        if not 'version' in obj:
            obj['version']=None
        return obj
    else:
        raise Exception('Failed to get master manifest ({})'.format(url))

def diffManifest(client,master):
    
    new = {
        'last_update': datetime.datetime.now().isoformat(),
        'version': master['version'],
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
    downloadFile(url,meta['tiny']['path'])
    downloadFile(url,meta['small']['path'])
    downloadFile(url,meta['medium']['path'])
    downloadFile(url,meta['large']['path'])
    downloadFile(url,meta['huge']['path'])


def fetchWebsite(url,shortTimeout):
    print('Cloning {}'.format(url))
    url = 'http://{}'.format(url)
    master_manifest = __get__manifest(url,shortTimeout)
    if not master_manifest['version']:
        raise Exception('Unknown master version, incompatible')
    elif master_manifest['version'] < supportedVersion:
        raise Exception("Master version {} is incompatible with this release {}".format(master_manifest['version'],supportedVersion))
    empty_manifest = {
        'last_update':'1970-01-01T00:00:01.000000',
        'host': os.getenv('HOSTNAME'),
         'img': {
            'inventory':[],
            'all':[],
            'new': [],
            'removed': []
        } 
    } 

    client_manifest = empty_manifest
    try:
        with open('api/manifest.json','r') as f:
            tmp=json.load(f)
            client_manifest=tmp
    except Exception:
        print('Slave node is clean, cloning everything')
    else:
        if client_manifest['version'] < supportedVersion:
            client_manifest = empty_manifest
            print('Stored client manifest is incompatible with current software version')
            print('WARN: Manifest ignored! Extra files might exist on disk that will not be deleted correctly')
    
    new_manifest = diffManifest(client_manifest,master_manifest)
    for meta in new_manifest['img']['removed']:
        removeMeta(meta) 

    for meta in new_manifest['img']['new']:
        cloneMeta(url,meta)
        with open(metaFilename(meta),'w') as f:
            json.dump(meta,f)
    return new_manifest
    
