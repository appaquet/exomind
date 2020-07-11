FROM node:alpine

ADD . /app/
WORKDIR /app/
RUN npm install && \
    npm run build

EXPOSE 8000
CMD ["npm", "start"]

