FROM node:16-slim

WORKDIR /code

COPY ./src/package.json ./src/package-lock.json ./

RUN npm install

COPY ./src ./

CMD ["node","index.js"]