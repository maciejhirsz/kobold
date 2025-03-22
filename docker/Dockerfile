FROM ubuntu:latest

WORKDIR /kobold
COPY . /kobold

ENV SHELL /bin/bash
# use ARG so not only set during build time
ARG DEBIAN_FRONTEND=noninteractive
ARG DEBCONF_NOWARNINGS="yes"

ENV USER=user

# https://dev.to/grigorkh/fix-tzdata-hangs-during-docker-image-build-4o9m
ENV TZ=Australia/Sydney
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

# install Docker is required to use Act
# https://docs.docker.com/desktop/install/linux-install/#kvm-virtualization-support
RUN echo "Running" \
  && apt-get update && apt-get upgrade -y \
  && apt-get install git vim curl unzip dialog -y \
  && useradd -m -u 2000 -U -s /bin/sh -d /$USER $USER \
  && chown -R $USER:$USER /kobold \
  && curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/nektos/act/master/install.sh | bash \
  && cp ./bin/act /usr/local/bin \
  && echo "Ready"

USER $USER
EXPOSE 5000
VOLUME ["/kobold"]
