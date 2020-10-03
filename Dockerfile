FROM nginx:mainline

RUN apt update && apt install -y python3 python3-pip exempi libcap2-bin   && \
    pip3 install jinja2 && \
    pip3 install Pillow && \
    pip3 install numpy && \
    pip3 install requests && \
    pip3 install python-xmp-toolkit && \
    mkdir -p /var/www/gallery && \ 
    cd /var/www/gallery && \
    mkdir tmp && \
    mkdir -p img/meta && \
    mkdir -p img/dropbox && \
    mkdir -p img/thumbnails  && \
    mkdir -p img/raw && \
    mkdir -p view && \
    mkdir -p api && \
    cd / && \
    useradd  gallery-owner  && \
    chown -R gallery-owner /var/www/gallery && \
    cd /var/www/gallery/tmp && \ 
    rm /docker-entrypoint.d/*.sh && \
    mkdir -p /var/run/nginx && \
    chown -R gallery-owner /var/cache/nginx && \
    chown -R gallery-owner /var/run/nginx && \
    chown -R gallery-owner /var/log/nginx && \
    setcap 'cap_net_bind_service=+ep' /usr/sbin/nginx 


COPY --chown=gallery-owner:gallery-owner website /var/www/gallery
COPY  docker/nginx.conf /etc/nginx/nginx.conf
COPY --chown=gallery-owner:gallery-owner docker/10-fetch-images.sh /docker-entrypoint.d/

USER gallery-owner
RUN chmod u+x /docker-entrypoint.d/10-fetch-images.sh
CMD ["nginx"]