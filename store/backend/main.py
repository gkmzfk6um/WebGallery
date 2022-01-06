import os
from flask import Flask, redirect
import gallery

def isReady():
    return True

app = Flask('gallerystore')
@app.route('/ready', methods=['GET'])
def routeIsReady():
    if gallery.isReady(app):
        return '',203
    else:
        return '',500 



if __name__== '__main__':
    app.run()