ROOT="/var/www/gallery" 
C=/opt/content-managment

if [ -n "$MASTER_NODE_URL" ]; then
    $C --root=$ROOT --clone-url="$MASTER_NODE_URL"
    if [ $? -ne 0 ]; then
        if [ -z "$DROPBOX_API_TOKEN" ]; then
            echo "Failed to clone $MASTER_NODE_URL [No other source available]";
            exit 1;
        else 
            echo "Failed to clone $MASTER_NODE_URL [Trying dropbox]";
        fi
    else 
        cloned=1
    fi
fi

if [ -n "$DROPBOX_API_TOKEN" ]; then
    $C --root=$ROOT --sync-dropbox
    if [ $? -ne 0  && $cloned -ne 1]; then
        echo "Failed to dropbox [No other source available]";
        exit 1
    fi
fi

$C --root=$ROOT  --generate && $C --root=$ROOT --manifest=yaml > /var/www/gallery/manifest.yaml