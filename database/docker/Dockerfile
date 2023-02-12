FROM postgres:15.1

ENV POSTGRES_USER shamebot
ENV POSTGRES_PASSWORD secret

COPY src/sql/init.sql /docker-entrypoint-initdb.d/

ENTRYPOINT [ "docker-entrypoint.sh" ]

EXPOSE 5432
CMD ["postgres"]
