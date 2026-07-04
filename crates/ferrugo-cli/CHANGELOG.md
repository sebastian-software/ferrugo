# Changelog

## [0.3.0](https://github.com/sebastian-software/ferrugo/compare/ferrugo-v0.2.0...ferrugo-v0.3.0) (2026-07-04)


### Features

* **native:** render simple optional content memberships ([9034c64](https://github.com/sebastian-software/ferrugo/commit/9034c64bcb78954ea0e70441e48dcf390573511f))
* **oracle:** add ghostscript matrix provider ([656d2a7](https://github.com/sebastian-software/ferrugo/commit/656d2a792f26c55a78c8ed67ae96244f44ac25da))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * ferrugo-native bumped from 0.2.0 to 0.3.0
    * ferrugo-pdfium bumped from 0.2.0 to 0.3.0
    * ferrugo-thumbnail bumped from 0.2.0 to 0.3.0

## [0.2.0](https://github.com/sebastian-software/ferrugo/compare/ferrugo-v0.1.0...ferrugo-v0.2.0) (2026-07-04)


### Performance Improvements

* **cli:** stream PNG filter rows into zlib ([c9e55c3](https://github.com/sebastian-software/ferrugo/commit/c9e55c3f0a0e697c447da827b3c360c03e9bb126))
* **native:** add scanned-page direct image route ([5c41bd5](https://github.com/sebastian-software/ferrugo/commit/5c41bd560e2f972ae105522ddada50e89f84332e))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * ferrugo-native bumped from 0.1.0 to 0.2.0
    * ferrugo-pdfium bumped from 0.1.0 to 0.2.0
    * ferrugo-thumbnail bumped from 0.1.0 to 0.2.0

## 0.1.0 (2026-07-03)


### Features

* add native document render sessions ([e277ee8](https://github.com/sebastian-software/ferrugo/commit/e277ee8f378fe85281950116d439ec8b5f547eed))
* add native render phase timings ([c1237c1](https://github.com/sebastian-software/ferrugo/commit/c1237c1295ac64887d2882f8934902f1a07b552b))
* add repeat render phase timings ([55a6f98](https://github.com/sebastian-software/ferrugo/commit/55a6f980b45498b6944379581226ef37d24f877b))
* add row bucket active trace counters ([0afd844](https://github.com/sebastian-software/ferrugo/commit/0afd8440eb9cdc7ab3dcfc6393b44dba24384b9b))
* add stroke shape trace diagnostics ([c9db435](https://github.com/sebastian-software/ferrugo/commit/c9db4354d754cce6ab2cde14feb4a89bb6a638ad))
* bound native document sessions ([53ee4e8](https://github.com/sebastian-software/ferrugo/commit/53ee4e882f11000a54fa4a1df473c75ce83cc5bf))
* **cli:** add renderer performance matrix ([c4fc125](https://github.com/sebastian-software/ferrugo/commit/c4fc125aaf2510e7e6a00d67ba04190896afb62b))
* **cli:** prepare ferrugo crate package ([f23871b](https://github.com/sebastian-software/ferrugo/commit/f23871b50bd499fc5ca2117e76ba3c527a28389f))
* enrich benchmark platform metadata ([c68cc41](https://github.com/sebastian-software/ferrugo/commit/c68cc41f8c130197a286f040f7146cd749b6717a))
* report benchmark timing reliability ([1b13626](https://github.com/sebastian-software/ferrugo/commit/1b136261efa0b20b48cbf3ce69275470936474fc))
* summarize repeat render phases ([6d09de3](https://github.com/sebastian-software/ferrugo/commit/6d09de3a40bcc177aa67c12cf0869e3dce0751eb))
* trace merged row bucket sample points ([14cd1af](https://github.com/sebastian-software/ferrugo/commit/14cd1af62dfb9425177abb4b0f63f0456b9a7f04))
* trace row bucket predicate work ([e34663a](https://github.com/sebastian-software/ferrugo/commit/e34663a92bbbc82da46403d2e9b52498763d9c47))
* trace span raster work ([0194519](https://github.com/sebastian-software/ferrugo/commit/01945198daf06d8f3ac757b83ccf28b20b875627))
* trace span route stroke counters ([8b3566b](https://github.com/sebastian-software/ferrugo/commit/8b3566b5c109727b63ad82b320aea6cf9d5c8179))
* trace stroke raster routes ([fc672d5](https://github.com/sebastian-software/ferrugo/commit/fc672d56a9c8c72312b43f8de55892674364737c))
* trace stroke raster routing counters ([455b433](https://github.com/sebastian-software/ferrugo/commit/455b433f38170e6d14f50dc35d745a30c17b6ec8))
* trace stroke row bucket work ([8383ae2](https://github.com/sebastian-software/ferrugo/commit/8383ae2cd9277370a7516c567f04823b1fbe2c6d))
* trace stroke span eligibility ([6eb5044](https://github.com/sebastian-software/ferrugo/commit/6eb5044937f26448cfcb8a13eb7e74209f7f70a7))


### Bug Fixes

* scope performance matrix to manifest ([16fba98](https://github.com/sebastian-software/ferrugo/commit/16fba98a422a83a14d6a183b512dc7991d2a0e0d))


### Performance Improvements

* attribute native resource decode phases ([4c439c6](https://github.com/sebastian-software/ferrugo/commit/4c439c61edfa479ed9f5ad99df046f9c7086f99f))
* cache decoded session image resources ([46d58da](https://github.com/sebastian-software/ferrugo/commit/46d58da3668b644edcd728e422e303676f73cf7b))
* **cli:** add default parallel scaling profiles ([fd9ebc1](https://github.com/sebastian-software/ferrugo/commit/fd9ebc1b5514e6d459124909f1a732b8a6d47250))
* **cli:** sample trace process rss ([7f36e83](https://github.com/sebastian-software/ferrugo/commit/7f36e83be13a8f8289001a8c491df3df892e0b0f))
* downsample low-memory image decode ([e169cb6](https://github.com/sebastian-software/ferrugo/commit/e169cb6224b71f0554e59e79133ddcf6a4606665))
* **native:** add opt-in parallel raster bands ([d95058e](https://github.com/sebastian-software/ferrugo/commit/d95058e5e7fc1809bde1bdcf39ac6a31e957033f))
* **native:** cache session font resources ([49c0119](https://github.com/sebastian-software/ferrugo/commit/49c01193a10db0dfc45ffa5ec630a1fd2bea06a9))
* **native:** replay safe renders in raster bands ([a754c09](https://github.com/sebastian-software/ferrugo/commit/a754c093f0858842cb38445ec32cc36b7ace8aa3))
* **native:** report raster band memory peaks ([c6ebcb4](https://github.com/sebastian-software/ferrugo/commit/c6ebcb42e3132067c3d7f371f9f8a101afed3077))
* **native:** retain icc transforms in document sessions ([00053e9](https://github.com/sebastian-software/ferrugo/commit/00053e97cd462d8e920a93423276372ad53f0af9))
* **native:** roll out thresholded default banding ([b8d47b1](https://github.com/sebastian-software/ferrugo/commit/b8d47b125671bd169a5e137127442ab7521bfbcf))
* **render:** adapt cubic curve flattening ([e15ff1e](https://github.com/sebastian-software/ferrugo/commit/e15ff1e6e5cadde8ddfaccc96eff61beb8b61084))
* **render:** add coverage span fill route ([54f6f7c](https://github.com/sebastian-software/ferrugo/commit/54f6f7c55b34ae03756da8a498f39c9909e4d284))
* **render:** add raster follow-up groundwork ([2fa3a52](https://github.com/sebastian-software/ferrugo/commit/2fa3a528503fa3b3ad05375710cdc31a664ab581))
* **render:** add scanline cell fill route ([dcb1a90](https://github.com/sebastian-software/ferrugo/commit/dcb1a906f97b79b9b155c5fec0300192f23337dd))
* **render:** buffer clipped edge coverage rows ([b2be303](https://github.com/sebastian-software/ferrugo/commit/b2be3030ef5403ec37daf9596c0a84a1f75c2134))
* **render:** cache strict type3 glyph renders ([23d7501](https://github.com/sebastian-software/ferrugo/commit/23d7501bea2ab2021aa337517bd0b34427c2cfbf))
* **render:** expose remaining stroke span routes ([89e1911](https://github.com/sebastian-software/ferrugo/commit/89e1911183c02c192d738475b158f1efbf7749d5))
* **render:** extend analytic coverage and stroke blitters ([809b141](https://github.com/sebastian-software/ferrugo/commit/809b141e06d3acb24ef9bb5c2ece6accf3bd9c7a))
* **render:** retain glyph bitmap cache in sessions ([edf2113](https://github.com/sebastian-software/ferrugo/commit/edf211302a6e818365593efc1577ec4232178515))
* **render:** retain type3 charproc templates in sessions ([78ef239](https://github.com/sebastian-software/ferrugo/commit/78ef239d8c3c78ce8ca9cd7ebdc0ccd15feb8e37))
* **render:** route joined strokes through fill outlines ([cf75474](https://github.com/sebastian-software/ferrugo/commit/cf7547461c2fddc9d5a2bfe6282bec29529bcf51))
* **render:** select fill span blitters per draw ([ae4ebb7](https://github.com/sebastian-software/ferrugo/commit/ae4ebb79466fed2f051f482b8b126546666af535))
* **render:** trace scanline alpha levels ([ee6fbb6](https://github.com/sebastian-software/ferrugo/commit/ee6fbb607cefb4314a45e3080f61e97627c6b306))
* trace image placement footprints ([49a86f6](https://github.com/sebastian-software/ferrugo/commit/49a86f607a9e5f947e1371393ad3a7a2094eb6da))
* trace image resource summaries ([9d4d8ab](https://github.com/sebastian-software/ferrugo/commit/9d4d8ab3a11e8b71144e8a404cd829a3e2d0b6ee))
