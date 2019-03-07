FROM ubuntu:trusty as ubuntu_trusty
RUN apt-get -y update
RUN apt-get -y install curl build-essential pkg-config libssl-dev
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
WORKDIR topiks
CMD bash -c 'source ~/.cargo/env && make package'

FROM centos:7 as centos_7
RUN yum -y groupinstall "Development Tools"
RUN yum -y install openssl-devel
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV SHA_COMMAND sha512sum
WORKDIR topiks
CMD bash -c 'source ~/.cargo/env && make package'