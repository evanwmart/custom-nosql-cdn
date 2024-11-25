# custom-nosql-cdn

### TODO List

1. [ ] **Enhance Logs Endpoint**
   - [ ] Store logs in memory or a file during runtime.
   - [ ] Serve these logs as plain text or JSON through the /logs route.

2. [ ] **Add Authentication**
   - [ ] Protect routes like /insert with an API key or token-based authentication.

3. [ ] **Scale for Concurrent Access**
   - [ ] Use a thread-safe data structure like Arc<Mutex<T>> to ensure safe concurrent access to the Database instance.

4. [ ] **Implement Tests**
   - [ ] Add unit tests and integration tests to ensure edge cases are handled correctly.

5. [ ] **Improve Efficiency**
   - [ ] Use a memory cache (e.g., HashMap) to speed up frequently accessed keys.

6. [ ] **Extend Data Structure**
   - [ ] Allow more complex values (e.g., nested structures or lists).
   - [ ] Introduce support for queries like "list all keys."
