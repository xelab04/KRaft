-- CREATE DATABASE kraft;

-- USE kraft;

-- CREATE TABLE users (
--     user_id int NOT NULL AUTO_INCREMENT,
--     username varchar(255) NOT NULL,
--     email varchar(255) NOT NULL,
--     password varchar(255) NOT NULL,
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

-- INSERT INTO users VALUES (1, "alex", "alexbissessur@gmail.com", "password");

-- INSERT INTO clusters VALUES (1, "test", "kraft.alexb.dev", 1);

IF NOT EXISTS (SELECT 1 FROM sys.databases WHERE name = 'kraft')
BEGIN
    CREATE DATABASE kraft;
END

GO
USE kraft;

GO
BEGIN
    CREATE TABLE users (
        user_id int NOT NULL AUTO_INCREMENT,
        username varchar(255) NOT NULL,
        email varchar(255) NOT NULL,
        password varchar(255) NOT NULL,
        PRIMARY KEY (user_id)
    );

    CREATE TABLE clusters (
        cluster_id int NOT NULL AUTO_INCREMENT,
        cluster_name varchar(255) NOT NULL,
        cluster_endpoint varchar(255) NOT NULL,
        user_id int NOT NULL,
        PRIMARY KEY (cluster_id),
        FOREIGN KEY (user_id) REFERENCES users(user_id)
    );
END;
