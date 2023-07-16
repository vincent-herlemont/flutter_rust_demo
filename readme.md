### Experiment of architecture for flutter app

Code:

- Share
    - [ ] Model type
    - [ ] WebSocket "transport" layer | Server <-> Client 
    - [ ] Http "command" layer | Server <-> Supabase | Server <-> Supabase
    - [ ] DB layer
    - [ ] Monitoring layer
- Server 
    - Sandalone
      - [ ] Create rust hub websockets part <-> Client
      - [ ] Create rust hub http part <-> Supabase
      - [ ] Create rust agent websockets part <-> hub
      - [ ] Create rust agent http part <-> Supabase
      - [ ] Create docker compose 
         - [x] Supabase [self-hosting/docker](https://supabase.com/docs/guides/self-hosting/docker)
         - [ ] Choose a proxy to reproduce the fly.io behaviour with fly-replay headers
         - [ ] Graphana 
  - Saas
     - [ ] Fly.io Draft real-time architecture
       * [Real-Time Collaboration with Replicache and Fly-Replay · Fly](https://fly.io/blog/replicache-machines-demo/)
       * [replicache-websocket/replicache-express/src/index.ts at 63cc00ad4875ce1a20780b7705ad72a4fd7c62f3 · fly-apps/replicache-websocket](https://github.com/fly-apps/replicache-websocket/blob/63cc00ad4875ce1a20780b7705ad72a4fd7c62f3/replicache-express/src/index.ts#L123)
       * [Dynamic Request Routing · Fly Docs](https://fly.io/docs/reference/dynamic-request-routing/#the-fly-prefer-region-request-header)
       * [Specify instance-id in fly-replay header - Questions / Help - Fly.io](https://community.fly.io/t/specify-instance-id-in-fly-replay-header/4869/10)
        - [ ] Check allow to deal with volumes (re-mount the volume to the right machine)
              Think: data is stored in the volume is the most important.
        - [ ] Check Instant snapshot of volume before migration?
     - [ ] Supabase (cloud)
     - [ ] Graphana (cloud)
- Client
    - [x] Flutter app
    - [ ] Setup rust bridge
- CI
    - [ ] Test flutter
    - [ ] Test rust
- CD

----
