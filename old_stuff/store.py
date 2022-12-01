import os
import json
import shutil
import copy
def findImage(inventory,name):
    for entry in inventory:
        if entry['name'] == name:
            return copy.deepcopy(entry)
    raise Exception("Store item {} not found in inventory".format(name))

def generateStore(inventory):

    apiPath = 'api/print'
    if os.path.isdir(apiPath):
        shutil.rmtree(apiPath)
    os.mkdir(apiPath)
    global storedata
    storedata = None
    if not storedata:
        if not(os.getenv("ENABLE_GALLERY_STORE")):
            return
        print("Enabling store!")
        with open('api/sitedata.json','r') as f:
            sitedata = json.load(f)
        storedata = sitedata["store"]
        for category,prints in storedata['prints'].items():
            for item in prints:
                item['data'] = findImage(inventory,item['name'])
                del item['data']['xmp']
                with open('api/print/{}.json'.format(item['data']['dropbox']['id']),'w') as f:
                    json.dump(item,f)

    return storedata 
    
