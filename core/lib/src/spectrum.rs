use crate::audio_clip::AudioClip;
use color_eyre::eyre::Result;
use realfft::RealFftPlanner;

pub fn spectrogram(clip: &AudioClip, mut range: (usize, usize)) -> Result<Vec<Vec<f32>>> {
    let resampled = clip.resample(12000);
    let n_fft = 2048;
    let offset = 200; // 16ms

    let signal = resampled.samples;
    let mut fft = RealFftPlanner::<f32>::new();
    let r2c = fft.plan_fft_forward(n_fft);
    let mut spectrums = Vec::new();
    let mut start_i = (range.0 / n_fft) * n_fft;
    let mut chunk = vec![0f32; n_fft];
    range.0 = range.0 * (resampled.sample_rate as usize) / (clip.sample_rate as usize);
    range.1 = range.1 * (resampled.sample_rate as usize) / (clip.sample_rate as usize);

    while start_i + n_fft < range.1 {
        for i in start_i..start_i + n_fft {
            chunk[i - start_i] = *signal.get(i).unwrap_or(&0f32);
        }

        // Hann window
        for (i, sample) in chunk.iter_mut().enumerate() {
            let x = (i as f32) / (n_fft as f32);
            *sample *= 0.5 * (1.0 - (2.0 * std::f32::consts::PI * x).cos());
        }

        // fft
        let mut spectrum = r2c.make_output_vec();
        r2c.process(&mut chunk, &mut spectrum)?;
        let mut spectrum = spectrum.to_vec();
        spectrum.truncate(n_fft / 2);
        spectrums.push(spectrum.into_iter().map(|x| x.re.abs()).collect());

        start_i += offset;
    }
    Ok(spectrums)
}

pub fn render_spectrogram(
    clip: &AudioClip,
    range: (usize, usize),
    width: usize,
    height: usize,
) -> Result<Vec<u8>> {
    let spectrums = spectrogram(clip, range)?;

    let pixels_per_spectrum = (width as f32) / (spectrums.len() as f32);
    let mut buffer = vec![0; width * height * 4];

    let min_freq = 1f32;
    let max_freq = 6000f32;

    for x in 0..width {
        let column = (x as f32) / pixels_per_spectrum;

        if let Some(spectrum) = spectrums.get(column as usize) {
            let mut prev_y = 0;
            let mut prev_num = 0f32;
            let mut prev_denom = 0f32;
            for (i, cell) in spectrum.iter().enumerate() {
                let frequency = (i as f32) * 12000f32 / 2048f32;
                let trans_min = 2595f32 * (1f32 + min_freq / 700f32).log10();
                let trans_max = 2595f32 * (1f32 + max_freq / 700f32).log10();
                let trans_val = 2595f32 * (1f32 + frequency / 700f32).log10();
                let coord = 2f32 * (trans_val - trans_min) / (trans_max - trans_min) - 1f32;
                let this_y = ((coord + 1.0) / 2.0 * (height as f32)).round() as usize;

                if this_y > prev_y {
                    while prev_y < this_y {
                        if prev_denom > 0.0 {
                            let color = COLORMAP[(255f32 * (prev_num / prev_denom)) as usize];
                            buffer[(height - prev_y - 1) * width * 4 + x * 4] =
                                (color[0] * 255.0) as u8;
                            buffer[(height - prev_y - 1) * width * 4 + x * 4 + 1] =
                                (color[1] * 255.0) as u8;
                            buffer[(height - prev_y - 1) * width * 4 + x * 4 + 2] =
                                (color[2] * 255.0) as u8;
                            buffer[(height - prev_y - 1) * width * 4 + x * 4 + 3] = 255;
                        }

                        prev_y += 1;
                    }
                    prev_num = 0f32;
                    prev_denom = 0f32;
                }

                let gain = 20f32 * cell.log10();
                let min_gain = -80f32;
                let max_gain = 20f32;
                let a = ((gain - min_gain) / (max_gain - min_gain)).clamp(0.0f32, 1.0f32);
                prev_num += a;
                prev_denom += 1f32;
            }
        }
    }

    Ok(buffer)
}

// acton colourmap https://github.com/tsipkens/cmap/blob/master/acton.m
// https://doi.org/10.5281/zenodo.1243862
const COLORMAP: [[f32; 3]; 256] = [
    [0.180_626_91, 0.129_915_98, 0.300_243_7],
    [0.184_609_89, 0.133_361_37, 0.303_782_5],
    [0.188_588_17, 0.136_829_14, 0.307_329_5],
    [0.192_546_78, 0.140_323_33, 0.310_900_24],
    [0.196_547_94, 0.143_831_66, 0.314_443_02],
    [0.200_488_48, 0.147_340_52, 0.318_014_7],
    [0.204_514_6, 0.150_846_35, 0.321_580_5],
    [0.208_493_22, 0.154_369_01, 0.325_153_4],
    [0.212_499_16, 0.157_916_04, 0.328_751_9],
    [0.216_523_45, 0.161_487_7, 0.332_345_5],
    [0.220_543_46, 0.164_996_95, 0.335_927_7],
    [0.224_525_66, 0.168_579_07, 0.339_540_27],
    [0.228_599_47, 0.172_137_96, 0.343_142_84],
    [0.232_626_91, 0.175_705_7, 0.346_748_98],
    [0.236_700_42, 0.179_308_92, 0.350_370_14],
    [0.240_737_56, 0.182_887_2, 0.353_977_98],
    [0.244_814_38, 0.186_501_56, 0.357_621_64],
    [0.248_929_55, 0.190_103_96, 0.361_237_26],
    [0.253_030_4, 0.193_733_16, 0.364_873_5],
    [0.257_149_28, 0.197_343_93, 0.368_523_33],
    [0.261_278_1, 0.200_942_31, 0.372_155_3],
    [0.265_425_18, 0.204_605_01, 0.375_810_06],
    [0.269_603_2, 0.208_226_44, 0.379_461_47],
    [0.273_782_73, 0.211_880_56, 0.383_123_55],
    [0.277_978_54, 0.215_504_94, 0.386_769],
    [0.282_204_8, 0.219_154_91, 0.390_434_5],
    [0.286_418_4, 0.222_807_35, 0.394_097_27],
    [0.290_685_86, 0.226_458_12, 0.397_768_26],
    [0.294_963_78, 0.230_076_77, 0.401_431_05],
    [0.299_282_22, 0.233_732_12, 0.405_094_5],
    [0.303_592_15, 0.237_416_46, 0.408_762_04],
    [0.307_941, 0.241_039_32, 0.412_435_7],
    [0.312_310_96, 0.244_674_95, 0.416_090_25],
    [0.316_729_13, 0.248_352_07, 0.419_755_43],
    [0.321_154_92, 0.251_982_96, 0.423_403_32],
    [0.325_598_1, 0.255_609_6, 0.427_068_9],
    [0.330_097_38, 0.259_251_15, 0.430_713_47],
    [0.334_616_42, 0.262_862_15, 0.434_348_9],
    [0.339_156_3, 0.266_473_5, 0.437_993_14],
    [0.343_714_65, 0.270_084_44, 0.441_615_97],
    [0.348_324_92, 0.273_672_7, 0.445_225_98],
    [0.352_962_37, 0.277_245_55, 0.448_835_9],
    [0.357_637_8, 0.280_787_05, 0.452_428_9],
    [0.362_328_1, 0.284_341_63, 0.455_993_98],
    [0.367_058_13, 0.287_868_1, 0.459_556_67],
    [0.371_818, 0.291_392_74, 0.463_085_47],
    [0.376_633_38, 0.294_860_87, 0.466_617_35],
    [0.381_461_62, 0.298_321_2, 0.470_118_88],
    [0.386_313_68, 0.301_752_5, 0.473_580_8],
    [0.391_226, 0.305_174_83, 0.477_034_8],
    [0.396_150_98, 0.308_549_23, 0.480_453_34],
    [0.401_118, 0.311_869_26, 0.483_837_87],
    [0.406_106_32, 0.315_186_6, 0.487_202_3],
    [0.411_120_8, 0.318_454_8, 0.490_523_64],
    [0.416_167_53, 0.321_674_02, 0.493_798_9],
    [0.421_235_92, 0.324_851_66, 0.497_038_96],
    [0.426_337_3, 0.327_991_9, 0.500_233],
    [0.431_472_8, 0.331_069_14, 0.503_383_3],
    [0.436_611_7, 0.334_121_4, 0.506_489_5],
    [0.441_778_33, 0.337_097_17, 0.509_546_5],
    [0.446_950_82, 0.340_021_46, 0.512_544_04],
    [0.452_153_6, 0.342_887_52, 0.515_484_45],
    [0.457_348_9, 0.345_671_98, 0.518_381],
    [0.462_560_6, 0.348_416_45, 0.521_200_36],
    [0.467_787_68, 0.351_091_03, 0.523_964_64],
    [0.473_007_02, 0.353_679_12, 0.526_669_14],
    [0.478_238_14, 0.356_220_7, 0.529_299_56],
    [0.483_458_46, 0.358_663_5, 0.531_857_6],
    [0.488_681_97, 0.361_046, 0.534_347],
    [0.493_892_16, 0.363_355_52, 0.536_776_7],
    [0.499_088_67, 0.365_588_72, 0.539_117_04],
    [0.504_283, 0.367_732_8, 0.541_384_4],
    [0.509_443_64, 0.369_805_4, 0.543_589_53],
    [0.514_595_3, 0.371_792_67, 0.545_715_6],
    [0.519_713_64, 0.373_712_33, 0.547_757_7],
    [0.524_818_5, 0.375_533_7, 0.549_728_33],
    [0.529_894_53, 0.377_288_37, 0.551_622_7],
    [0.534_937_2, 0.378_951_58, 0.553_430_56],
    [0.539_947_57, 0.380_543_92, 0.555_172_74],
    [0.544_933_14, 0.382_040_62, 0.556_844_06],
    [0.549_88, 0.383_479_54, 0.558_422_6],
    [0.554_790_2, 0.384_818_05, 0.559_949_34],
    [0.559_666_2, 0.386_089_5, 0.561_388_4],
    [0.564_499_5, 0.387_290_42, 0.562_767_7],
    [0.569_293_6, 0.388_424_75, 0.564_069_7],
    [0.574_053_9, 0.389_477_55, 0.565_293_2],
    [0.578_778_15, 0.390_462_7, 0.566_472_8],
    [0.583_457_65, 0.391_391_37, 0.567_578_85],
    [0.588_098_94, 0.392_240_26, 0.568_619_6],
    [0.592_717_3, 0.393_034_6, 0.569_605_5],
    [0.597_284_2, 0.393_772_24, 0.570_537_7],
    [0.601_813_5, 0.394_455_6, 0.571_418_3],
    [0.606_312_6, 0.395_091_1, 0.572_252],
    [0.610_785_9, 0.395_679_24, 0.573_036_2],
    [0.615_216_26, 0.396_213_56, 0.573_769_2],
    [0.619_633_8, 0.396_704_4, 0.574_462_83],
    [0.624_015_4, 0.397_167_98, 0.575_128_9],
    [0.628_373_15, 0.397_605_36, 0.575_766_8],
    [0.632_707_1, 0.398_012_04, 0.576_369_76],
    [0.637_028_34, 0.398_390_44, 0.576_941_13],
    [0.641_324_76, 0.398_748_5, 0.577_489_55],
    [0.645_618_4, 0.399_093_87, 0.578_027_07],
    [0.649_908_24, 0.399_434_63, 0.578_558_9],
    [0.654_185_2, 0.399_779_02, 0.579_085_77],
    [0.658_464_25, 0.400_134_8, 0.579_609_2],
    [0.662_752_1, 0.400_503_75, 0.580_132_25],
    [0.667_036_24, 0.400_884_93, 0.580_662_85],
    [0.671_339_3, 0.401_281_24, 0.581_210_2],
    [0.675_657_75, 0.401_707_95, 0.581_780_1],
    [0.679_990_9, 0.402_184_72, 0.582_378_1],
    [0.684_352_16, 0.402_720_4, 0.583_011_15],
    [0.688_750_57, 0.403_316_26, 0.583_686_05],
    [0.693_169_7, 0.403_970_48, 0.584_410_1],
    [0.697_618_4, 0.404_696_7, 0.585_192_1],
    [0.702_108_74, 0.405_530_3, 0.586_029_9],
    [0.706_646_26, 0.406_447_65, 0.586_932_4],
    [0.711_215_56, 0.407_470_02, 0.587_915_06],
    [0.715_826_75, 0.408_614_52, 0.588_983_06],
    [0.720_468_5, 0.409_892_05, 0.590_144_63],
    [0.725_159_94, 0.411_292_02, 0.591_406_6],
    [0.729_876_4, 0.412_854_94, 0.592_771_65],
    [0.734_623_13, 0.414_551_2, 0.594_233_3],
    [0.739_394_6, 0.416_419_18, 0.595_813_4],
    [0.744_168_64, 0.418_443_9, 0.597_513_44],
    [0.748_950_3, 0.420_647_92, 0.599_331_5],
    [0.753_727_73, 0.423_021_44, 0.601_262_2],
    [0.758_483_95, 0.425_579_04, 0.603_322_45],
    [0.763_196_6, 0.428_302_35, 0.605_490_57],
    [0.767_860_7, 0.431_199_25, 0.607_790_4],
    [0.772_459_75, 0.434_245_88, 0.610_184_7],
    [0.776_976_5, 0.437_474_04, 0.612_689_3],
    [0.781_381_9, 0.440_823_2, 0.615_295_2],
    [0.785_670_1, 0.444_338_08, 0.617_993_35],
    [0.789_818_94, 0.447_950_5, 0.620_776],
    [0.793_821_2, 0.451_697_86, 0.623_623_97],
    [0.797_645_2, 0.455_528_23, 0.626_524_75],
    [0.801_296_35, 0.459_445_12, 0.629_480_06],
    [0.804_757_3, 0.463_415_32, 0.632_478_8],
    [0.808_016_24, 0.467_454_58, 0.635_497_87],
    [0.811_068_06, 0.471_521_7, 0.638_538_3],
    [0.813_906_5, 0.475_606_68, 0.641_580_34],
    [0.816_538_93, 0.479_697_17, 0.644_622_2],
    [0.818_947_1, 0.483_773_14, 0.647_655_67],
    [0.821_153_4, 0.487_855_55, 0.650_670_95],
    [0.823_149_5, 0.491_897_7, 0.653_657_1],
    [0.824_942_47, 0.495_898_07, 0.656_620_86],
    [0.826_546_6, 0.499_866_93, 0.659_540_7],
    [0.827_957_6, 0.503_800_45, 0.662_431_8],
    [0.829_202_3, 0.507_681_25, 0.665_272_53],
    [0.830_272_14, 0.511_500_66, 0.668_084_1],
    [0.831_190_35, 0.515_275_54, 0.670_847_4],
    [0.831_962_05, 0.519_007_27, 0.673_567_6],
    [0.832_595_17, 0.522_672_1, 0.676_250_2],
    [0.833_109_26, 0.526_298_3, 0.678_890_1],
    [0.833_513_26, 0.529_868_25, 0.681_492_4],
    [0.833_814, 0.533_389_1, 0.684_062_2],
    [0.834_021_2, 0.536_885_3, 0.686_602_4],
    [0.834_145_25, 0.540_322_3, 0.689_112_66],
    [0.834_195_6, 0.543_731_45, 0.691_582_9],
    [0.834_181_1, 0.547_106_27, 0.694_027_9],
    [0.834_109_6, 0.550_453_07, 0.696_460_84],
    [0.833_988_55, 0.553_768_9, 0.698_860_35],
    [0.833_824_75, 0.557_074_1, 0.701_248_35],
    [0.833_624_5, 0.560_345_6, 0.703_619_5],
    [0.833_393_9, 0.563_608_8, 0.705_975_6],
    [0.833_139_06, 0.566_852_5, 0.708_318_6],
    [0.832_866_25, 0.570_077_3, 0.710_651_46],
    [0.832_581_16, 0.573_310_6, 0.712_971_2],
    [0.832_287_9, 0.576_534_7, 0.715_295_1],
    [0.831_989_47, 0.579_747_56, 0.717_610_6],
    [0.831_688_64, 0.582_952_44, 0.719_917_83],
    [0.831_388_8, 0.586_172_6, 0.722_220_66],
    [0.831_093_97, 0.589_380_3, 0.724_534_1],
    [0.830_807_86, 0.592_613_1, 0.726_840_9],
    [0.830_533_6, 0.595_830_7, 0.729_151_7],
    [0.830_274, 0.599_066_6, 0.731_464_27],
    [0.830_031_2, 0.602_303_7, 0.733_783_96],
    [0.829_807_46, 0.605_550_35, 0.736_101],
    [0.829_604_6, 0.608_823, 0.738_432_17],
    [0.829_424_4, 0.612_085_34, 0.740_763_8],
    [0.829_268_4, 0.615_368_7, 0.743_110_84],
    [0.829_138_3, 0.618_676_1, 0.745_453_36],
    [0.829_035_6, 0.621_976_5, 0.747_813_76],
    [0.828_961_73, 0.625_307_5, 0.750_172_3],
    [0.828_918_1, 0.628_653_05, 0.752_542_6],
    [0.828_905_8, 0.632_007_8, 0.754_931_15],
    [0.828_926, 0.635_377_94, 0.757_317_3],
    [0.828_979_55, 0.638_769_57, 0.759_713_77],
    [0.829_067_05, 0.642_177_7, 0.762_124_06],
    [0.829_189_24, 0.645_596_15, 0.764_546_45],
    [0.829_346_5, 0.649_043_8, 0.766_972_6],
    [0.829_539_36, 0.652_496, 0.769_410_9],
    [0.829_768_3, 0.655_978, 0.771_864_1],
    [0.830_034, 0.659_465_7, 0.774_318_4],
    [0.830_337_1, 0.662_980_6, 0.776_794_3],
    [0.830_678, 0.666_499_8, 0.779_268_5],
    [0.831_057_1, 0.670_054_1, 0.781_761_4],
    [0.831_473_77, 0.673_611_76, 0.784_261_7],
    [0.831_926_2, 0.677_196_15, 0.786_77],
    [0.832_413_2, 0.680_781_9, 0.789_285_96],
    [0.832_938_85, 0.684_395_73, 0.791_810_1],
    [0.833_506_35, 0.688_031_6, 0.794_350_15],
    [0.834_109_2, 0.691_677_03, 0.796_890_9],
    [0.834_744_6, 0.695_339_3, 0.799_452_4],
    [0.835_417_3, 0.699_013_65, 0.802_010_36],
    [0.836_126_5, 0.702_712_1, 0.804_585_93],
    [0.836_872, 0.706_423_3, 0.807_167_23],
    [0.837_653, 0.710_144_6, 0.809_754_4],
    [0.838_461_7, 0.713_881_1, 0.812_349_56],
    [0.839_310_6, 0.717_638_7, 0.814_954_6],
    [0.840_191_84, 0.721_397_16, 0.817_566_2],
    [0.841_100_75, 0.725_185_4, 0.820_185],
    [0.842_045_07, 0.728_973, 0.822_815_7],
    [0.843_015_9, 0.732_783_2, 0.825_446_4],
    [0.844_017_15, 0.736_596_76, 0.828_086_73],
    [0.845_051_47, 0.740_429_64, 0.830_737_7],
    [0.846_108_56, 0.744_271_34, 0.833_390_1],
    [0.847_196_2, 0.748_128_4, 0.836_048_7],
    [0.848_311_66, 0.751_991_87, 0.838_712_1],
    [0.849_455_2, 0.755_866_65, 0.841_385_66],
    [0.850_615, 0.759_748_46, 0.844_059_6],
    [0.851_805_75, 0.763_645_9, 0.846_739_9],
    [0.853_016_85, 0.767_550_3, 0.849_431_9],
    [0.854_248_8, 0.771_471_44, 0.852_124_3],
    [0.855_507_55, 0.775_390_4, 0.854_819_2],
    [0.856_778_44, 0.779_324, 0.857_516_7],
    [0.858_072_1, 0.783_265_35, 0.860_226_33],
    [0.859_386_8, 0.787_217_6, 0.862_937_15],
    [0.860_717, 0.791_171_43, 0.865_646_8],
    [0.862_067_94, 0.795_136_3, 0.868_370_3],
    [0.863_430_74, 0.799_111_6, 0.871_086_5],
    [0.864_804_15, 0.803_085_9, 0.873_811_2],
    [0.866_198_54, 0.807_075_7, 0.876_537_9],
    [0.867_604_14, 0.811_065_4, 0.879_271_15],
    [0.869_025_4, 0.815_062_64, 0.882_008_7],
    [0.870_448_8, 0.819_064_26, 0.884_748],
    [0.871_896_5, 0.823_078, 0.887_486_8],
    [0.873_344_66, 0.827_095_3, 0.890_233],
    [0.874_805_57, 0.831_117_9, 0.892_981],
    [0.876_275_1, 0.835_141_54, 0.895_732_76],
    [0.877_755_1, 0.839_172_84, 0.898_483_04],
    [0.879_238_25, 0.843_207_2, 0.901_239_93],
    [0.880_731_5, 0.847_246_47, 0.903_997_1],
    [0.882_229_8, 0.851_290_3, 0.906_760_3],
    [0.883_737_44, 0.855_346_26, 0.909_521_76],
    [0.885_244_2, 0.859_397, 0.912_287_06],
    [0.886_758_74, 0.863_457_7, 0.915_052_65],
    [0.888_274_2, 0.867_515_1, 0.917_826_7],
    [0.889_792_6, 0.871_582_1, 0.920_591_7],
    [0.891_315, 0.875_649_4, 0.923_371],
    [0.892_84, 0.879_717_77, 0.926_144_2],
    [0.894_367_8, 0.883_796_9, 0.928_918_8],
    [0.895_892_56, 0.887_870_4, 0.931_698_8],
    [0.897_421_3, 0.891_954_3, 0.934_479],
    [0.898_946_05, 0.896_037_16, 0.937_266],
    [0.900_471_75, 0.900_123_1, 0.940_051_14],
];
