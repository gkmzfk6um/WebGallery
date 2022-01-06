import os
import requests
from requests.adapters import HTTPAdapter
from requests.packages.urllib3.util.retry import Retry


adapter = HTTPAdapter(max_retries=Retry(total=5, backoff_factor=0.5,status_forcelist=[403,429,500, 502, 503, 504] ) )
http = requests.Session()
http.mount("https://", adapter)
http.mount("http://", adapter)

galleryUrl = os.getenv("GALLERY_URL")


supported = 5
def isReady(app):
    fetchUrl = '{}/api/manifest.json'.format(galleryUrl)
    r = http.get(fetchUrl)
    if r.ok:
        obj = r.json()
        if not 'version' in obj:
            app.logger.warn("Gallery API didn't provided version, assuming incompatibility")
            return False
        elif obj['version'] < supported:
            app.logger.warn("Gallery API incompatible. API v{} < supported v{}".format(obj['version'],supported))
            return False
    else:
        app.logger.warn('Failled to connect to Gallery')
        return False
    return True
        
        