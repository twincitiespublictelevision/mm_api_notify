FROM debian:buster

COPY mm_api_notify /usr/sbin/mm_api_notify
COPY init.d/mm_api_notify.example /etc/init.d/mm_api_notify
RUN chmod +x /etc/init.d/mm_api_notify