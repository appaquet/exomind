# Exomind

![Build](https://github.com/appaquet/exomind/workflows/Push%20tester/badge.svg)
![iOS app](https://build.appcenter.ms/v0.1/apps/76ac2c48-f34c-4ac4-bcc4-41bae61f8177/branches/app-build/badge)

**Warning: Exomind is at a very early development stage, hence incomplete, unstable and probably totally unsafe. Use at your own risk.**

Exomind is a personal knowledge management tool. It is a unified inbox in which your emails, your notes, your tasks and your bookmarks can live
and be organized in collections.

It is built on top of [Exocore](../exocore), a distributed application framework, and is meant to be hosted in a
decentralized fashion on user's selected servers (ex: Raspberry Pi, VPS, etc.).

## Roadmap

Exomind closely follows [Exocore's roadmap](../exocore#roadmap) since Exocore is being developed for Exomind.

### v0.1 (in progress)

* **Notes, Bookmarks, Emails, Tasks**
* **Snoozing**
* **Basic web client**
* **Basic iOS client**
* **Gmail synchronization server** (read-only, except for labels assignations and read flags)
* **WASM business logic** (hosted by Exocore)

### v0.2

* **Gmail attachments**

### v0.3 and beyond

* **File storage**

## Apps

<img src="https://user-images.githubusercontent.com/129552/107126442-fb39c500-687d-11eb-8e61-39d66a3edf3d.gif" height="350" />   <img src="https://user-images.githubusercontent.com/129552/107126280-e6a8fd00-687c-11eb-9a00-5e2405bfcc59.gif" height="350" />

## Dependencies

* Install dependencies from [Exocore](../exocore) and follow web and iOS.

## Quick start

1. Bootstrap an exocore node. (see [Exocore's quick start](../exocore#quick-start))
   If you already have an Exocore cluster, make sure one node has the `app_host` role.
    * `exo node init`
    * `exo cell init`

2. Install Exomind in the cell.
    * `exo cell app install https://github.com/appaquet/exomind/releases/download/<VERSION>/exomind-app.zip`

3. Start your node.
    * `exo daemon`

4. Download pre-built Electron app, or follow [Web client](./web/README.md) and/or [iOS client](./ios/README.md) instructions.

## Usage

* See [Gmail integration README](./integrations/gmail/README.md)

* See [Web / Electron README](./web/README.md)

* See [iOS README](./ios/README.md)

* See [Browser extensions README](./browsers/README.md)
