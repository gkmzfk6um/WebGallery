ARG SOURCE_COMMIT=DEV            
FROM node:current-buster-slim as website-build
ARG SOURCE_COMMIT                  
ENV SOURCE_COMMIT $SOURCE_COMMIT 

RUN apt update && apt install -y python3 python3-pip && \
    pip3 install Flask && \
    pip3 install jinja2 && \
    pip3 install Stripe && \
    pip3 install gunicorn && \
    cd / && \
    useradd  store-owner  && \
    mkdir -p /var/www/gallery-store &&\
    chown -R store-owner /var/www/gallery-store && \
    cd /var/www/gallery-store 
COPY backend/ /var/www/gallery-store

USER store-owner
WORKDIR /var/www/gallery-store
CMD gunicorn --bind 0.0.0.0:4280  main:app