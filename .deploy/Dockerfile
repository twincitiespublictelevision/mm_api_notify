FROM centos:centos6.9

COPY mm_api_notify /usr/sbin/mm_api_notify
COPY init.d/mm_api_notify.example /etc/init.d/mm_api_notify
COPY config.sh /var/run/config.sh

CMD ["/bin/bash", "/var/run/config.sh"]