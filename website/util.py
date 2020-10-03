import os
import re

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

def removePathNoFail(path):
    try:
        os.remove(path)
    except Exception:
        print('Failed to remove path ({})'.format(path))

def metaFilename(meta):
    return  "img/meta/{}.json".format(meta['dropbox']['id'])

def dropboxFilenameFromRawPath(path):
    filename = os.path.basename(path)
    name = os.path.splitext(filename)[0]
    return  "img/dropbox/{}.json".format(name)

    

                    
def removeMeta(meta):    
    print('Deleting all data of img ({})'.format(meta['name']))
    removePathNoFail(meta['original']['path'])
    removePathNoFail(meta['tiny']['path'])
    removePathNoFail(meta['small']['path'])
    removePathNoFail(meta['original']['path'])
    removePathNoFail(meta['view'])
    removePathNoFail(toJsonPath(meta['view']))
    os.remove(metaFilename(meta))

#https://stackoverflow.com/questions/1094841/reusable-library-to-get-human-readable-version-of-file-size
def sizeof_fmt(num, suffix='B'):
    for unit in ['','Ki','Mi','Gi','Ti','Pi','Ei','Zi']:
        if abs(num) < 1024.0:
            return "%3.1f%s%s" % (num, unit, suffix)
        num /= 1024.0
    return "%.1f%s%s" % (num, 'Yi', suffix)