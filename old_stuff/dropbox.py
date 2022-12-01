import requests
import json
import re
import shutil
import os
import base64
from util import *

def __handle__error(j):
    if 'error' in j:
        if 'error_summery' in j:
            raise Exception(j['error_summery'])
        else:
            raise Exception(j['error'])

def __get_page(token,cursor):
    url = "https://api.dropboxapi.com/2/files/list_folder/continue"
    headers = {
        "Authorization": "Bearer {}".format(token),
        "Content-Type": "application/json"
    }
    data = {
        "cursor": cursor
    }
    r = requests.post(url, headers=headers, data=json.dumps(data))
    if r.ok:
        j = r.json()
        __handle__error(j)
        yield from __yield__files(j ,token)

def __yield__files(j,token):
        for e in j['entries']:
            if e['.tag'] == 'file':
                if e['is_downloadable']:
                    yield {
                        'name': e['name'],
                        'id': e['id'],
                        'id_stripped':  re.sub('id: ','',e['id']),
                        'path': e['path_lower'],
                        'content_hash': e['content_hash']
                    }
                else:
                    print("Warning file {} isn't downloadable".format(e['name']))
        if j['has_more']:
           yield from __get_page(token,j['cursor'])


def getFileMeta(token):
    url = "https://api.dropboxapi.com/2/files/list_folder"

    headers = {
        "Authorization": "Bearer {}".format(token),
        "Content-Type": "application/json"
    }

    data = {
        "path": "/Photography/Published",
        "recursive": False,
        "include_media_info": False,
        "include_deleted": False,
        "limit": 200
    }

    r = requests.post(url, headers=headers, data=json.dumps(data))
    if r.ok:
        j = r.json()
        __handle__error(j)
        yield from __yield__files(j ,token)
    else:
        print(r)
        print(r.text)
        print(r.json())
        raise Exception('Failed to request file information')


def downloadFile(dropbox,token):
    
    url = "https://content.dropboxapi.com/2/files/download"
    headers = {
        "Authorization": "Bearer {}".format(token),
        "Dropbox-API-Arg": "{\"path\":\""+dropbox['id']+"\"}"
    }
    r = requests.post(url, headers=headers,stream=True)
    if r.ok:
        r.raw.decode_content=True
        match = re.match("([^/\\\\]+)\.([jJ][pP][gG])",dropbox['name'])

        if match:
            filename = "img/raw/{}.jpg".format(match[1])
            if match[2] != "jpg":
                print("Renamed {} to {}.jpg".format(dropbox['name'],match[1]))
            with open(filename,'wb') as f:
                shutil.copyfileobj(r.raw,f)
            print("Downloaded {} ({})".format(dropbox['name'],sizeof_fmt(os.path.getsize(filename))))
        else:
            raise Exception("File extension of ({}) ins't valid, expected .jpg".format(dropbox['name']))
    else:
        print("Failed to download {} from dropbox".format(dropbox['name']))
        print(r)
        print(r.text)
        print(r.json())
        raise Exception('Failed to request file information')

def downloadSitedata(dropbox,token):
    url = "https://content.dropboxapi.com/2/files/download"
    headers = {
        "Authorization": "Bearer {}".format(token),
        "Dropbox-API-Arg": "{\"path\":\""+dropbox['id']+"\"}"
    }
    r = requests.post(url, headers=headers,stream=True)
    if r.ok:
        r.raw.decode_content=True
        match = re.match("sitedata\.json",dropbox['name'])

        if match:
            filename = "api/sitedata.json"
            with open(filename,'wb') as f:
                shutil.copyfileobj(r.raw,f)
            print("Downloaded {} ({})".format(dropbox['name'],sizeof_fmt(os.path.getsize(filename))))
        else:
            raise Exception("Expected sitedata.json not {}".format(dropbox['name']))
    else:
        print("Failed to download {} from dropbox".format(dropbox['name']))
        print(r)
        print(r.text)
        print(r.json())
        raise Exception('Failed to request file information')