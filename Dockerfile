ARG SOURCE_COMMIT=DEV            
FROM node:current-buster-slim as website-build
ARG SOURCE_COMMIT                  
ENV SOURCE_COMMIT $SOURCE_COMMIT 
WORKDIR /build 
RUN     apt update && apt install -y rename \
     && npm install --save-dev @babel/cli @babel/core @babel/preset-env babel-preset-minify sass
ENV BROWSERSLIST "> 0.5%, last 2 versions, Firefox ESR, not dead" 
COPY website website
RUN mv  website/js website/jssrc  \
    && ./node_modules/.bin/babel website/jssrc --out-dir website/js --source-maps --presets=@babel/preset-env,minify \
    && ./node_modules/.bin/sass  website/sass:website/css --style=compressed --color \
    &&  rename -v "s/(.*)\.css/\1-${SOURCE_COMMIT}.css/" website/css/*.css


FROM nginx:mainline
ARG SOURCE_COMMIT                  
ENV SOURCE_COMMIT $SOURCE_COMMIT 

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


COPY --from=website-build --chown=gallery-owner:gallery-owner /build/website /var/www/gallery
COPY  docker/nginx.conf /etc/nginx/nginx.conf
COPY --chown=gallery-owner:gallery-owner docker/10-fetch-images.sh /docker-entrypoint.d/

USER gallery-owner
RUN chmod u+x /docker-entrypoint.d/10-fetch-images.sh && \
    echo "{\"git\":\"$SOURCE_COMMIT\"}" > /var/www/gallery/version.json && \
    chmod 400 /var/www/gallery/version.json 
CMD ["nginx"]