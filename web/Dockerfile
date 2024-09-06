FROM nginx:latest

RUN echo "Building for platform $TARGETPLATFORM arch $TARGETARCH"

ADD nginx.conf /etc/nginx/nginx.conf
ADD dist/ /etc/nginx/html/
