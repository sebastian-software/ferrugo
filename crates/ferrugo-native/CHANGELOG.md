# Changelog

## 0.1.0 (2026-07-03)


### Features

* add native document render sessions ([e277ee8](https://github.com/sebastian-software/ferrugo/commit/e277ee8f378fe85281950116d439ec8b5f547eed))
* add native render phase timings ([c1237c1](https://github.com/sebastian-software/ferrugo/commit/c1237c1295ac64887d2882f8934902f1a07b552b))
* add repeat render phase timings ([55a6f98](https://github.com/sebastian-software/ferrugo/commit/55a6f980b45498b6944379581226ef37d24f877b))
* add stroke shape trace diagnostics ([c9db435](https://github.com/sebastian-software/ferrugo/commit/c9db4354d754cce6ab2cde14feb4a89bb6a638ad))
* bound native document sessions ([53ee4e8](https://github.com/sebastian-software/ferrugo/commit/53ee4e882f11000a54fa4a1df473c75ce83cc5bf))
* trace stroke raster routes ([fc672d5](https://github.com/sebastian-software/ferrugo/commit/fc672d56a9c8c72312b43f8de55892674364737c))


### Bug Fixes

* **render:** stabilize coverage and banded stroke routes ([f3f0c80](https://github.com/sebastian-software/ferrugo/commit/f3f0c80aca686ff957634fa75778bd06f7b0a493))


### Performance Improvements

* attribute native resource decode phases ([4c439c6](https://github.com/sebastian-software/ferrugo/commit/4c439c61edfa479ed9f5ad99df046f9c7086f99f))
* attribute ordered image raster timing ([d50905c](https://github.com/sebastian-software/ferrugo/commit/d50905c5d700e9862a4134e2c17a62ca772e72f7))
* cache decoded session image resources ([46d58da](https://github.com/sebastian-software/ferrugo/commit/46d58da3668b644edcd728e422e303676f73cf7b))
* downsample low-memory image decode ([e169cb6](https://github.com/sebastian-software/ferrugo/commit/e169cb6224b71f0554e59e79133ddcf6a4606665))
* **native:** add opt-in parallel raster bands ([d95058e](https://github.com/sebastian-software/ferrugo/commit/d95058e5e7fc1809bde1bdcf39ac6a31e957033f))
* **native:** band alpha transparency replay ([c2ec93b](https://github.com/sebastian-software/ferrugo/commit/c2ec93ba08d04ff05e76d29c2aa7c7422daf0c1a))
* **native:** band axis-aligned line joins ([e2cf4bc](https://github.com/sebastian-software/ferrugo/commit/e2cf4bc96f2eef9d1c63f64713000d849f0ca4f1))
* **native:** band cubic round vector paths ([50f8df1](https://github.com/sebastian-software/ferrugo/commit/50f8df1be47992e99c51ea9473c4e3f331ab5295))
* **native:** band dashed map overlays ([ad495f3](https://github.com/sebastian-software/ferrugo/commit/ad495f3fa6915d58aadfa855b8588971c5a82aa8))
* **native:** band independent line strokes ([6424e13](https://github.com/sebastian-software/ferrugo/commit/6424e13c802889f06a863d2b508e7b5becfd513d))
* **native:** band isolated knockout groups ([3d911a1](https://github.com/sebastian-software/ferrugo/commit/3d911a1e4cdf88156b23ef0467e18a75b6c4eb11))
* **native:** band joined-outline stroke replay ([92c2a9d](https://github.com/sebastian-software/ferrugo/commit/92c2a9db88c21fe819465423b4a5580833d5b70b))
* **native:** band line-only clipped path replay ([2d6d75e](https://github.com/sebastian-software/ferrugo/commit/2d6d75eb8b81d3b8c5ed5e09966f93855afd1945))
* **native:** band multi-subpath line clips ([89f5a0d](https://github.com/sebastian-software/ferrugo/commit/89f5a0d55c637068f54d2a18ff5b369e5c3af0d6))
* **native:** band shading display replay ([898f1b1](https://github.com/sebastian-software/ferrugo/commit/898f1b113a3183e7bff53b1e3d93031aa2b15320))
* **native:** band simple fill path raster work ([984323e](https://github.com/sebastian-software/ferrugo/commit/984323ee897eff6d50994e8042e74bee333b7aa9))
* **native:** band simple fill-stroke path replay ([be5a560](https://github.com/sebastian-software/ferrugo/commit/be5a56060dc2ec4caa6d214c124de1ced22374a9))
* **native:** band simple stroke path replay ([73e26d8](https://github.com/sebastian-software/ferrugo/commit/73e26d88b41c8f2798ed928bc0f6adeef320ef9a))
* **native:** band small dashed stroke groups ([374e8ac](https://github.com/sebastian-software/ferrugo/commit/374e8acec48c303db0d961e06b216014343f8936))
* **native:** band solid line-cap strokes ([0aef54d](https://github.com/sebastian-software/ferrugo/commit/0aef54d954b6b497e3c019c69d783fe146a2eda7))
* **native:** band technical hatch strokes ([3458b03](https://github.com/sebastian-software/ferrugo/commit/3458b030fce2fdd605b5df1e69e4a1aacb9be8b2))
* **native:** band tiling-pattern fill replay ([0f4f60f](https://github.com/sebastian-software/ferrugo/commit/0f4f60f13012730e7f7af739f86959b1a32c262d))
* **native:** band transparency group replay ([569e2ee](https://github.com/sebastian-software/ferrugo/commit/569e2ee3e3f0d1f0d177cfd82e172010c2ad2c11))
* **native:** cache session font resources ([49c0119](https://github.com/sebastian-software/ferrugo/commit/49c01193a10db0dfc45ffa5ec630a1fd2bea06a9))
* **native:** replay safe renders in raster bands ([a754c09](https://github.com/sebastian-software/ferrugo/commit/a754c093f0858842cb38445ec32cc36b7ace8aa3))
* **native:** report raster band memory peaks ([c6ebcb4](https://github.com/sebastian-software/ferrugo/commit/c6ebcb42e3132067c3d7f371f9f8a101afed3077))
* **native:** retain icc transforms in document sessions ([00053e9](https://github.com/sebastian-software/ferrugo/commit/00053e97cd462d8e920a93423276372ad53f0af9))
* **native:** roll out thresholded default banding ([b8d47b1](https://github.com/sebastian-software/ferrugo/commit/b8d47b125671bd169a5e137127442ab7521bfbcf))
* **native:** share font resources across pages ([4741772](https://github.com/sebastian-software/ferrugo/commit/4741772a0941f8e5f31241f3ec8eb82e8c948d02))
* **render:** adapt cubic curve flattening ([e15ff1e](https://github.com/sebastian-software/ferrugo/commit/e15ff1e6e5cadde8ddfaccc96eff61beb8b61084))
* **render:** add coverage span fill route ([54f6f7c](https://github.com/sebastian-software/ferrugo/commit/54f6f7c55b34ae03756da8a498f39c9909e4d284))
* **render:** add raster follow-up groundwork ([2fa3a52](https://github.com/sebastian-software/ferrugo/commit/2fa3a528503fa3b3ad05375710cdc31a664ab581))
* **render:** cache strict type3 glyph renders ([23d7501](https://github.com/sebastian-software/ferrugo/commit/23d7501bea2ab2021aa337517bd0b34427c2cfbf))
* **render:** extend analytic coverage and stroke blitters ([809b141](https://github.com/sebastian-software/ferrugo/commit/809b141e06d3acb24ef9bb5c2ece6accf3bd9c7a))
* **render:** retain glyph bitmap cache in sessions ([edf2113](https://github.com/sebastian-software/ferrugo/commit/edf211302a6e818365593efc1577ec4232178515))
* **render:** retain type3 charproc templates in sessions ([78ef239](https://github.com/sebastian-software/ferrugo/commit/78ef239d8c3c78ce8ca9cd7ebdc0ccd15feb8e37))
* **render:** retire stroke distance routes ([7c7ef40](https://github.com/sebastian-software/ferrugo/commit/7c7ef407fcc3a6148b673d81c019452a0efd2a3b))
* reserve flate image decode output ([0b539f6](https://github.com/sebastian-software/ferrugo/commit/0b539f609e66307efba8fbac152d78584edb567f))
* trace image placement footprints ([49a86f6](https://github.com/sebastian-software/ferrugo/commit/49a86f607a9e5f947e1371393ad3a7a2094eb6da))
* trace image resource summaries ([9d4d8ab](https://github.com/sebastian-software/ferrugo/commit/9d4d8ab3a11e8b71144e8a404cd829a3e2d0b6ee))
