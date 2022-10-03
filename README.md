# Learning about idempotency tokens

The idempotency token is a unique token that, when provided, is used in the HTTP request de-duplication process and plays a vital role in developing _idempotent APIs_.

The _idempotency token_ alongside the hash of part (or all) of the request creates a unique identifier for a given request. Suppose the API consumer provides the same _idempotency token_ alongside the same request body. In that case, we will return the previous response (or an error in the case if the last request did not finish yet).

Of course, one has to manage the TTL of the responses. Otherwise, you might create a situation where making new resources is impossible!
