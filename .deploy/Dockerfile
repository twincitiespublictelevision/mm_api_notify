FROM debian:buster

RUN apt-get update
RUN apt-get install -y supervisor

RUN mkdir /var/log/mm_api_notify

COPY mm_api_notify /usr/sbin/mm_api_notify
COPY mm_api_notify.conf /etc/supervisor/conf.d/mm_api_notify.conf