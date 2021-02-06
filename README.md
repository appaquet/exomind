# Exomind

**Warning: Exomind is at a very early development stage, hence incomplete, unstable and probably totally unsafe. Use at your own risk.**

Exomind is personal knowledge management tool. It is an unified inbox in which your emails, your notes, your tasks and your bookmarks can live
and be organized in collections. 

It is built on top of [Exocore](https://github.com/appaquet/exocore), a distributed application framework, and is meant to be hosted in a 
decentralized fashion on user's selected servers (ex: Raspberry Pi, VPS, etc.).
## Roadmap
Exomind closely follows [Exocore's roadmap](https://github.com/appaquet/exocore#roadmap) since Exocore is being developed for Exomind. 

### v0.1 (in progress)
* **Notes, Bookmarks, Emails, Tasks**
* **Snoozing**
* **Basic web client**
* **Basic iOS client**
* **Gmail synchronization server** (read-only, except for labels assignations and read flags)
* **Business logic server** (i.e. not hosted by Exocore)

### v0.2
* **WASM business logic** (replaces the business logic server, hosted in Exocore)
* **Gmail attachments**

### v0.3 and beyond
* **Node discovery** (i.e. no manual cell's nodes configuration)
* **File storage**

## Apps
<img src="https://user-images.githubusercontent.com/129552/107126442-fb39c500-687d-11eb-8e61-39d66a3edf3d.gif" height="350" />   <img src="https://user-images.githubusercontent.com/129552/107126280-e6a8fd00-687c-11eb-9a00-5e2405bfcc59.gif" height="350" />

## Dependencies
* Install dependencies from [Exocore](https://github.com/appaquet/exocore) and follow web and iOS.

## Usage

* See [Server README](./server/README.md)

* See [Gmail integration README](./integrations/gmail/README.md)

* See [Web / Electron README](./web/README.md)

* See [iOS README](./ios/README.md)

* See [Browser extensions README](./browsers/README.md)
