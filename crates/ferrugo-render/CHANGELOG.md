# Changelog

## 0.1.0 (2026-07-01)


### Features

* add row bucket active trace counters ([0afd844](https://github.com/sebastian-software/ferrugo/commit/0afd8440eb9cdc7ab3dcfc6393b44dba24384b9b))
* add stroke shape trace diagnostics ([c9db435](https://github.com/sebastian-software/ferrugo/commit/c9db4354d754cce6ab2cde14feb4a89bb6a638ad))
* trace merged row bucket sample points ([14cd1af](https://github.com/sebastian-software/ferrugo/commit/14cd1af62dfb9425177abb4b0f63f0456b9a7f04))
* trace row bucket predicate work ([e34663a](https://github.com/sebastian-software/ferrugo/commit/e34663a92bbbc82da46403d2e9b52498763d9c47))
* trace span raster work ([0194519](https://github.com/sebastian-software/ferrugo/commit/01945198daf06d8f3ac757b83ccf28b20b875627))
* trace span route stroke counters ([8b3566b](https://github.com/sebastian-software/ferrugo/commit/8b3566b5c109727b63ad82b320aea6cf9d5c8179))
* trace stroke raster routes ([fc672d5](https://github.com/sebastian-software/ferrugo/commit/fc672d56a9c8c72312b43f8de55892674364737c))
* trace stroke raster routing counters ([455b433](https://github.com/sebastian-software/ferrugo/commit/455b433f38170e6d14f50dc35d745a30c17b6ec8))
* trace stroke row bucket work ([8383ae2](https://github.com/sebastian-software/ferrugo/commit/8383ae2cd9277370a7516c567f04823b1fbe2c6d))
* trace stroke span eligibility ([6eb5044](https://github.com/sebastian-software/ferrugo/commit/6eb5044937f26448cfcb8a13eb7e74209f7f70a7))


### Bug Fixes

* continue row bucket stroke scans after x misses ([b5e93c6](https://github.com/sebastian-software/ferrugo/commit/b5e93c626847633efcc5e6e6cdb4514828a9a724))
* cull off-device paths before flattening ([81036bf](https://github.com/sebastian-software/ferrugo/commit/81036bfd3c07309957ecf598fc756b5209b34f3a))


### Performance Improvements

* advance axial shading by row ([45f2a0c](https://github.com/sebastian-software/ferrugo/commit/45f2a0c51ce6c4a65ca5621c3eb1a552521a4e40))
* advance span coverage with row cursors ([e84c5f6](https://github.com/sebastian-software/ferrugo/commit/e84c5f646974c8b25a87007a922f54c97110720b))
* attribute ordered image raster timing ([d50905c](https://github.com/sebastian-software/ferrugo/commit/d50905c5d700e9862a4134e2c17a62ca772e72f7))
* bound prepared stroke joins ([01cfd12](https://github.com/sebastian-software/ferrugo/commit/01cfd1296c57031624865b57da7812ab86b99a3b))
* bound rectangle fills by rect clips ([bad55a1](https://github.com/sebastian-software/ferrugo/commit/bad55a1c8e970bf028e95d42237c578ea8f16e42))
* bucket axis stroke joins ([7cb4de8](https://github.com/sebastian-software/ferrugo/commit/7cb4de8620d4cd8eb4e26cfe76b1b6294d23e24e))
* bucket stroke joins by row ([887184a](https://github.com/sebastian-software/ferrugo/commit/887184a7e40fbba5f8fe8dfd8919a8a09a7a1cd0))
* bucket stroke lines by raster row ([0e14303](https://github.com/sebastian-software/ferrugo/commit/0e1430313a7695fa5d598fc467ed40d17ee52097))
* build axis spans in two passes ([ceb5154](https://github.com/sebastian-software/ferrugo/commit/ceb515403e1c5745116b1441a6affa7980a3beee))
* cull stroke segment distance checks ([4440c74](https://github.com/sebastian-software/ferrugo/commit/4440c74160c73e28c101688a55a4a578ed91e4d9))
* downsample low-memory image decode ([e169cb6](https://github.com/sebastian-software/ferrugo/commit/e169cb6224b71f0554e59e79133ddcf6a4606665))
* fast path axial shading rows ([d999f15](https://github.com/sebastian-software/ferrugo/commit/d999f15655f0e705c75c5518896dd73778e0f1c5))
* fast path common shading exponents ([3495c52](https://github.com/sebastian-software/ferrugo/commit/3495c5255666cefd06058ead3d976dafda46d557))
* fast path radial shading ([c704556](https://github.com/sebastian-software/ferrugo/commit/c7045567c101e6779ffddca483e79cec08328d59))
* fast path rectangle clip predicates ([d4660ea](https://github.com/sebastian-software/ferrugo/commit/d4660ea2231fddfcd44ce4c31f00a1d732ce4d6a))
* fast path tiling pattern rects ([0b57600](https://github.com/sebastian-software/ferrugo/commit/0b576005a5f6581ca235a9c9715e1558cc69783b))
* fast-path opaque gray image sampling ([3d3d35f](https://github.com/sebastian-software/ferrugo/commit/3d3d35fd51fe0b3c2bdd941796434bf92007cbe0))
* fast-path opaque image interior writes ([4d93337](https://github.com/sebastian-software/ferrugo/commit/4d93337a9066eb789ca553c60bf46fc17b8ba25f))
* fast-path opaque rgb image sampling ([38a9d99](https://github.com/sebastian-software/ferrugo/commit/38a9d996dc82737cd47cbba731041057b9fd160e))
* fill opaque rects by row ([373b8d9](https://github.com/sebastian-software/ferrugo/commit/373b8d97ae8d2414d0856563e19d7df0405dfc47))
* fill text rectangles by row ([8293ca6](https://github.com/sebastian-software/ferrugo/commit/8293ca60c4a63be91b112eca073e816a4aa86ff6))
* flatten joined axis span raster builder ([6779a17](https://github.com/sebastian-software/ferrugo/commit/6779a174cfb5449181d3dd5d5a4c9773fa7cfb71))
* flatten stroke bucket indices ([5100761](https://github.com/sebastian-software/ferrugo/commit/51007612e39f1a878733ad771dd4e1417f3c68ae))
* gate active row-bucket stroke scans ([672c8aa](https://github.com/sebastian-software/ferrugo/commit/672c8aad1a5215e608e356a1aea04be3be5a6d9a))
* intersect raster bounds with active clips ([587ff7a](https://github.com/sebastian-software/ferrugo/commit/587ff7a2e7491630105c7ccdd2dfcfc727d582a5))
* lower axis stroke span threshold ([9648814](https://github.com/sebastian-software/ferrugo/commit/9648814bfb71cfce9fc9fc99eefa8a2f00636791))
* pre-sort row bucket candidates ([f5ca0d7](https://github.com/sebastian-software/ferrugo/commit/f5ca0d72f71ad32400e95bd6ca8c0d6924c0a11f))
* precompute row bucket line metrics ([b29d608](https://github.com/sebastian-software/ferrugo/commit/b29d6084499cf59587c5cba77dc99485ab62823a))
* precompute stroke join geometry ([f153498](https://github.com/sebastian-software/ferrugo/commit/f153498734eed902bc6bc9c6ac3c212721adf873))
* rasterize axis strokes sparsely ([ba2cd27](https://github.com/sebastian-software/ferrugo/commit/ba2cd271b9a8795865e5bebdcb46b9e1f8b9bb8b))
* rasterize axis strokes with spans ([33e1592](https://github.com/sebastian-software/ferrugo/commit/33e15921543ba5c84a1a2e8e76a961f7bc48bf7c))
* rasterize exact axis spans ([440130e](https://github.com/sebastian-software/ferrugo/commit/440130ec7f1fb416048866354517fadb74fd9e66))
* rasterize row bucket ranges ([b1e0fdf](https://github.com/sebastian-software/ferrugo/commit/b1e0fdf1c124b0977dfce3e38bc7b633afab528e))
* reserve flate image decode output ([0b539f6](https://github.com/sebastian-software/ferrugo/commit/0b539f609e66307efba8fbac152d78584edb567f))
* route axis lines through spans ([381a891](https://github.com/sebastian-software/ferrugo/commit/381a8918b68f5d20a64f19a0957948d66eabfc9d))
* share decoded image sample vectors ([3407ae5](https://github.com/sebastian-software/ferrugo/commit/3407ae5d81fae6afb3ebe5ac961fb3a071f6b710))
* short-circuit opaque normal blends ([76572cf](https://github.com/sebastian-software/ferrugo/commit/76572cfe76fe8234d962311f8b818e14940dce20))
* shortcut opaque normal blending ([4b9a957](https://github.com/sebastian-software/ferrugo/commit/4b9a957fcc176260529b6a47ae3ff023b990c82d))
* skip empty stroke joins ([6fc94dd](https://github.com/sebastian-software/ferrugo/commit/6fc94dd2f1c12da26d7eed057810949ef14dcc53))
* skip pixel-aligned rect clip checks ([2dd0999](https://github.com/sebastian-software/ferrugo/commit/2dd09999607cdf932c9c93c5f45f76f2ac33611f))
* span-rasterize diagonal strokes ([35ed279](https://github.com/sebastian-software/ferrugo/commit/35ed279c025905b3fc4df6e30ab5bc096f0151f2))
* speed up axis-aligned image rasterization ([d8085b4](https://github.com/sebastian-software/ferrugo/commit/d8085b499ce12e6c7156768402d0588fa96c5f03))
* stream fallback text rasterization ([0c8de0b](https://github.com/sebastian-software/ferrugo/commit/0c8de0b04eefb49e7e6dda9dcc95a25d6d984db2))
* trace image placement footprints ([49a86f6](https://github.com/sebastian-software/ferrugo/commit/49a86f607a9e5f947e1371393ad3a7a2094eb6da))
* trace image resource summaries ([9d4d8ab](https://github.com/sebastian-software/ferrugo/commit/9d4d8ab3a11e8b71144e8a404cd829a3e2d0b6ee))
* write axis-aligned images by row ([f85b26b](https://github.com/sebastian-software/ferrugo/commit/f85b26ba0f17a9c5ecf5fec07818cb000f87b1c9))
* write blended pixels with one offset ([c3d0f18](https://github.com/sebastian-software/ferrugo/commit/c3d0f188a619fcb754ecce2b51aae35aa73b6300))
