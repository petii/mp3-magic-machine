# M<sup>3</sup> $\rightarrow$ the MP3 Magic Machine
A learning project which also is hopefully solving a real problem (at least for me.)

**Background:**
band practice recordings (recorded by the drummer) are shared in WAV which takes up way too much space. I took the responsibility upon myself to convert these to MP3s which take up significantly less space.

**Problem statement:**
doing this weekly is inconvinient.

**Proposed solution:**
blob storage in the cloud with a serverless function triggering off of new files and doing the conversions.

**Plus points and learning opportunities:**
  - putting my time spent learning rust to the test
  - learn about and get more confortable with GHA and IaC

## Project status

- [x] automatic deployment of lambda from main branch
- [x] reading S3 object into WavReader
- [x] saving resulting files back into blob storage
- [ ] use better path structure
- [ ] create zip of mp3s and upload that into blob storage
- [ ] performance investigations: currently a ~400MB WAV uses about 1.8GB of memory and runs for ~2 minutes.
      - usual load would be 2 such files at once
      - memory goes up to 3008MB
      - timeout can be increased to 15 minutes
- [ ] figure out "public" S3 upload & download
- [ ] figure out how to move S3 and lambda config to repo
