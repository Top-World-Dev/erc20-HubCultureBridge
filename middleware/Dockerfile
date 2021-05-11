FROM ubuntu:18.04

RUN apt-get update

RUN export DEBIAN_FRONTEND=noninteractive && \
    apt-get install -y tzdata && \
    ln -fs /usr/share/zoneinfo/America/New_York /etc/localtime && \
    dpkg-reconfigure --frontend noninteractive tzdata

RUN apt-get install -y vim\
    python3-pip \
    nano

COPY requirements.txt requirements.txt

RUN pip3 install -r requirements.txt && rm requirements.txt

COPY middleware.py middleware.py

ENTRYPOINT ["python3","-u","middleware.py"]

