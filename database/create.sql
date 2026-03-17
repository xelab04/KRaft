-- CREATE DATABASE kraft;

-- USE kraft;

-- CREATE TABLE users (
--     user_id int NOT NULL AUTO_INCREMENT,
--     uuid varchar(255) NOT NULL,
--     username varchar(255) NOT NULL,
--     email varchar(255) NOT NULL,
--     password varchar(255) NOT NULL,
--     betacode varchar(255),
--     admin BOOLEAN NOT NULL,
--     verified_email BOOLEAN NOT NULL,
--     last_email datetime,
--     verification_code varchar(255),
--     PRIMARY KEY (user_id)
-- );

-- CREATE TABLE clusters (
--     cluster_id int NOT NULL AUTO_INCREMENT,
--     cluster_name varchar(255) NOT NULL,
--     cluster_endpoint varchar(255) NOT NULL,
--     user_id int NOT NULL,
--     PRIMARY KEY (cluster_id),
--     FOREIGN KEY (user_id) REFERENCES users(user_id)
-- );

-- CREATE TABLE betacode (
--     betacode varchar(255) NOT NULL,
--     enabled BOOLEAN NOT NULL DEFAULT FALSE
-- );

-- INSERT INTO users VALUES (1, "alex", "alexbissessur@gmail.com", "password");

-- INSERT INTO clusters VALUES (1, "test", "kraft.alexb.dev", 1);


-- \c kraft

CREATE TABLE users (
    user_id SERIAL PRIMARY KEY,
    uuid VARCHAR(255) NOT NULL,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    password VARCHAR(255) NOT NULL,
    betacode VARCHAR(255),
    admin BOOLEAN NOT NULL,
    verified_email BOOLEAN NOT NULL,
    last_email TIMESTAMP,
    verification_code VARCHAR(255)
);

CREATE TABLE clusters (
    cluster_id SERIAL PRIMARY KEY,
    cluster_name VARCHAR(255) NOT NULL,
    cluster_endpoint VARCHAR(255) NOT NULL,
    user_id INT NOT NULL,
    CONSTRAINT fk_user
        FOREIGN KEY (user_id)
        REFERENCES users(user_id)
);

CREATE TABLE betacode (
    betacode VARCHAR(255) PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT FALSE
);
