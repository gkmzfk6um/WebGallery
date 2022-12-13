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

FROM node:current-buster-slim as rust-website-build
ARG SOURCE_COMMIT                  
ENV SOURCE_COMMIT $SOURCE_COMMIT 
WORKDIR /build 
RUN  apt update && apt install -y curl build-essential openssl pkg-config libssl-dev git && curl  --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

COPY content-managment/Cargo.toml content-managment/Cargo.toml
COPY content-managment/main/Cargo.toml content-managment/main/Cargo.toml
COPY content-managment/datamodel/Cargo.toml content-managment/datamodel/Cargo.toml

RUN  mkdir -p content-managment/main/src  \
    && mkdir -p content-managment/datamodel/src  \
    && echo "// dummy file" > content-managment/main/src/lib.rs  \
    && echo "// dummy file" > content-managment/datamodel/src/lib.rs  \
    && . "$HOME/.cargo/env" \ 
    && cd content-managment \
    && cargo build --release 
RUN    rm content-managment/main/src/lib.rs \
    && rm content-managment/datamodel/src/lib.rs 
COPY content-managment content-managment
RUN . "$HOME/.cargo/env" \ 
    && cd content-managment \
    &&  touch datamodel/src/lib.rs \
    &&  cargo build --release


FROM nginx:mainline
ARG SOURCE_COMMIT                  
ENV SOURCE_COMMIT $SOURCE_COMMIT 

RUN apt update && apt install -y  libcap2-bin && \
    mkdir -p /var/www/gallery && \ 
    cd /var/www/gallery && \
    mkdir tmp && \
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
COPY --from=rust-website-build --chown=gallery-owner:gallery-owner /build/content-managment/target/release/content-managment /opt/
COPY  docker/nginx.conf /etc/nginx/nginx.conf
COPY --chown=gallery-owner:gallery-owner docker/10-fetch-images.sh /docker-entrypoint.d/

USER gallery-owner
RUN chmod u+x /docker-entrypoint.d/10-fetch-images.sh && \
    echo "{\"git\":\"$SOURCE_COMMIT\"}" > /var/www/gallery/version.json && \
    chmod 400 /var/www/gallery/version.json && \
    /opt/content-managment --create-dir --root=/var/www/gallery --print-id='.*'
CMD ["nginx"]