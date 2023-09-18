CREATE TABLE subscription_tokens(
    subscription_token TEXT NOT NULL,
    subscriber_id uuid NOT NULL
    /* Foreign key assures there is an entry in subscriptions table for every token: */
    REFERENCES subscriptions (id), 
    PRIMARY KEY (subscription_token)
);