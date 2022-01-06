#categories = {
#    "Analog" : [
#        {
#            "mode": "or",
#            "filters": [
#                {"namespace": "http://ns.adobe.com/exif/1.0/aux/", "path": "aux:Lens", "value": "65.0 mm" },
#                {"path": "dc:subject", "value": "Negative" } ,
#                {"path": "dc:subject", "value": "Negative Lab Pro" },
#                {"path": "dc:subject", "value": "Film" } 
#            ]
#        }
#    ],
#    "Digital": [ 
#        {
#            "mode": "or",
#            "filters": [
#                {"namespace": "http://ns.adobe.com/exif/1.0/aux/", "path": "aux:Lens", "value": "Fujifilm Fujinon XF23mmF2 R WR" },
#                {"namespace": "http://ns.adobe.com/exif/1.0/aux/", "path": "aux:Lens", "value": "XF23mmF2 R WR" },
#                {"namespace": "http://ns.adobe.com/exif/1.0/aux/", "path": "aux:Lens", "value": "Fujifilm Fujinon XF50mmF2 R WR" },
#                {"namespace": "http://ns.adobe.com/exif/1.0/aux/", "path": "aux:Lens", "value": "XF50mmF2 R WR" },
#                {"namespace": "http://ns.adobe.com/exif/1.0/aux/", "path": "aux:Lens", "value": "Fujifilm Fujinon XF90mmF2 R LM WR" },
#                {"namespace": "http://ns.adobe.com/exif/1.0/aux/", "path": "aux:Lens", "value": "XF90mmF2 R LM WR" },
#                {"namespace": "http://ns.adobe.com/exif/1.0/aux/", "path": "aux:Lens", "value": "Fujifilm Fujinon XF18-55mmF2.8-4 R LM OIS" },
#                {"namespace": "http://ns.adobe.com/exif/1.0/aux/", "path": "aux:Lens", "value": "XF18-55mmF2.8-4 R LM OIS" },
#                {"path": "dc:subject", "value": "Digital" } 
#            ]
#        }
#    ],
#    "Portraiture" : [
#        {
#            "mode": "or",
#            "filters": [
#                {"path": "dc:subject", "value": "Portrait" } 
#            ]
#        }
#    ],
#    "Black and white" : [
#        {
#            "mode": "or",
#            "filters": [
#                {"path": "dc:subject", "value": "BnW"  },
#                {"path": "dc:subject", "value": "B&W" }, 
#                {"path": "dc:subject", "value": "BNW" } 
#            ]
#        }
#    ],
#    "Nighttime" : [
#        {
#            "mode": "or",
#            "filters": [
#                {"path": "dc:subject", "value": "Night"  }
#            ]
#        }
#    ]
#}

from PIL import Image
from libxmp.utils import object_to_dict
from libxmp import XMPMeta
from libxmp.consts import XMP_NS_DC
from util import *
import re
import json

def loadXmp(inventory):
    for item in inventory:
        if 'xmp' in item:
            try:
                yield (item, object_to_dict(XMPMeta(xmp_str=item['xmp'])))
            except OSError:
                print("!!!WARN Failed to deserialize XMP data in {}".format(item['name']))
                continue
        else:
            print("!!!WARN: File metadata entry doesn't contain serialized xmp data, ignoring item from categories")

def loadAndValidateCategories():
    global categories
    with open('api/sitedata.json') as f:
        sitedata = json.load(f)
        categories = sitedata['categories']

    failed = False
    def warn(w):
        nonlocal failed
        print("!!!WARN: validateCategories(): {}".format(w))
        failed = True
    if not (type(categories) is dict):
        warn("Categories must be a dictionary!")
        return False

    isString = lambda x : type(x) is str

    prefix = ""
    for catName,cat in categories.items():
        if type(cat) is list:
            if len(cat) == 0:
                warn("Category {} must contain at least one filter definition".format(catName))
            for i in range(len(cat)):
                filterdef = cat[i]
                if type(filterdef) is dict:
                    for key,value in filterdef.items():
                        if key == "mode":
                            if not (value in ("or","and")):
                                warn("Value \"{}\" isn't a valid mode, expected [or,and]".format())
                        elif key == "filters":
                            if type(value) is list:
                                if len(value) == 0:
                                    warn("attribute \"filters\" must contains at least one filter!")
                                for j in range(len(value)):
                                    filter = value[j] 
                                    if type(filter) is dict:
                                        for filterkey,filtervalue in filter.items():
                                            if not filterkey in ("namespace","regex","path","value","debug"):
                                                warn("Unknown attribute \"{}\" in filter[{}] of filterdefinition[{}] of {}, expected [namespace,regex,path,value]".format(filterkey,j,i,catName))
                                            if not isString(filtervalue):
                                                warn("Attribute \"{}\" in filter[{}] of filterdefinition[{}] of {} must have type string".format(filterkey,j,i,catName))
                                        if not "path" in filter:
                                            warn("Filter[{}] of filterdefinition[{}] of {} must contain attribute \"path\"".format(filterkey,j,i,catName))
                                        if not ("regex" in filter or "value" in filter):
                                            warn("Filter[{}] of filterdefinition[{}] of {} must contain attribute regex or value!".format(filterkey,j,i,catName))
                                    else:
                                        warn("filter[{}] of filterdefinition[{}] of {} must be of type dict!",j,i,catName)
                            else:
                                warn("property \"filters\" of {} filterdefinition[{}] must be list of filter!".format(catName,i))
                        else:
                            warn("\"{}\" isn't a valid filterdefinition property, expected [filters,mode]".format(key))
                    if not "filters" in filterdef:
                        warn("Filter definition must contain property \"filters\"")
                else:
                    warn("Type of filter definition {} filter[{}] ins't dictionary".format(catName,i))
        else:
            warn("Type of category \"{}\" isn't list of filter!")
    return not failed


xmpMetadata = {}
def matchFilter(filter,xmp):
    global xmpMetadata
    namespace = XMP_NS_DC
    if "namespace" in filter:
        namespace = filter["namespace"]
    
    debug = "debug" in filter
    # Collect debug data
    if debug:
        for ns,t  in xmp.items():
            if not (ns in xmpMetadata):
                xmpMetadata[ns] = {}
            for key,val,p in xmp[ns]:
                if not (key in xmpMetadata[ns]):
                    xmpMetadata[ns][key]=0
                xmpMetadata[ns][key] = xmpMetadata[ns][key]+1

    def match(filter,value):
        if "regex" in filter and re.match(filter["regex"],value):
            return True
        elif "value" in filter and filter["value"]==value:
            return True
        return False


    # Do the match
    if namespace in xmp:
        ns = xmp[namespace]
        for path,value,p in ns:
            if path == filter["path"]:
                if debug:
                    print("## DEBUG {} ##".format(path))
                    print("\tvalue: \"{}\"".format(value))
                    for prop,val in p.items():
                        if val:
                            print("\tproperty: {}=True".format(prop))
                if p["VALUE_IS_ARRAY"]:
                    aPath = lambda i:  "{}[{}]".format(path,i)
                    i = 1 
                    for ipath,aValue,_ in ns:
                        if ipath == aPath(i):
                            if debug:
                                print("\tvalue[{}]: {}".format(i,aValue))

                            if match(filter,aValue):
                                return True    
                            i=i+1
                else:
                    if match(filter,value):
                        return True
    return False



def sortbycategories(inventory):
    if not ('sortedinventory' in globals()):
        global sortedinventory 
        sortedinventory = {}
        for category, filter in categories.items():
           sortedinventory[category]=[]
        debugTarget="debug"
        for item, xmp in loadXmp(inventory):
            debug = debugTarget == item['displayname']
            if debug:
                for x,y in xmp.items():
                    print(x)
                    for a in y:
                        print("\t{}: {}".format(a[0],a[1]))
            for category, filterdefinitions in categories.items():
                if debug:
                    print("Matching cat {}".format(category))
                isMatch = False
                for filterdefinition in filterdefinitions:
                    mode = lambda x,y : x or y
                    isMatch = False
                    if "mode" in filterdefinition:
                        if filterdefinition["mode"] == "and":
                            isMatch = True
                            mode = lambda x,y : x and y 

                    for filter in filterdefinition["filters"]:
                        if debug:
                            print("Trying filter {}".format(filter))
                        isMatch = mode(isMatch,matchFilter(filter,xmp)  )  

                    if isMatch:
                        if debug:
                            print("Matched {}".format(category))
                        sortedinventory[category].append(item)
                        break
    
        print("Generated categories: ")
        sortedinventoryItems = list(sortedinventory.items())
        emptyCat = False
        for i in range(len(sortedinventoryItems)):
            name,imgs = sortedinventoryItems[i] 
            print("({}) {}: {}".format(i,name,len(imgs)))
            if len(imgs) == 0:
                emptyCat= True
                del sortedinventory[name]

        if emptyCat:
            print("!!!WARN: Category empty, dumping xmp-debug.json")
            with open("xmp-debug.json","w") as f:
                json.dump(xmpMetadata,f)
    return sortedinventory

