FROM registry.opensuse.org/opensuse/nginx:latest

COPY ./index.html /srv/www/htdocs

COPY ./sub /srv/www/htdocs/sub

COPY ./assets /srv/www/htdocs/assets
RUN rm /srv/www/htdocs/assets/88x31.gif

COPY ./nginx.conf /etc/nginx/nginx.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
