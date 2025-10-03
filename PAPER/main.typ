#import "shortcut.typ": header, tab
#import "abstruct_model.typ": section

#set page(
  paper: "a4",
  numbering: "1",
  columns: 2,
)

#set text(
  font: "TH Sarabun New",
  size: 16pt,
)

#place(
  top + center,
  float: true,
  scope: "parent",
  clearance: 2em,
)[
  #par(justify: false)[
    #text(size: 20pt)[*การหาแนวโน้มของตลาดหุ้น S&P 500*] \
    #text(size: 18pt)[
      ปริญญา อบอุ่น \
      ภาควิชาวิศวกรรมคอมพิวเตอร์ คณะวิศวกรรมศาสตร์ มหาวิทยาลัยเกษตรศาสตร์
    ]
  ]
]

#section("บทคัดย่อ")[
  โปรเจคนี้นำเสนอเกี่ยวกับระบบหาโน้มโน้มของตลาดหุ้นดัชนี S&P 500 โดยใช้เทคนิคการประมวลผลสัญญานดิจิตัล ได้แก่ เส้นค่าเฉลี่ยนเคลื่อนที่ (Moving Average) เส้นค่าเฉลี่ยเคลื่อนที่แบบเอ็กซ์โพเนนเชียล (EMA) และ model Auto-Regressive Integrated Moving Average สำหรับการพยากรณ์ทิศทางตลาด การศึกษานี้ใช้ข้อมูลปิดรายวันของดัชนี S&P 500 ตั้งแต่ปี 2015-2025 จำนวนข้อมูล 2,516 จุด โดยข้อมูลจะผ่านการทดสอบ Stationarity ด้วย ADF และ KPSS test และหาค่า parameter ที่เหมาะสม Grid Search 100x100 combination จากการทดลองพบว่า EMA(11) ตัดขึ้น SMA(99)  EMA(11) x SMA(99) ให้ประสิทธิภาพสูงสุดที่ Accuracy 53.45% Precision 54.90% และ Recall 77.24% ในส่วนของ EMA ตัดกับ SMA ในส่วนของ EMA(11) ตัดกับ EMA(99) ให้ผลที่ดีกว่าสำหรับ EMAfast ตัดกับ EMAslow คือ Accuracy 52.83% และ Precision 54.35% และ Recall 78.77% และที่โมเดล ARIMA(1,0,2)
  ให้ Accuracy 52.19%  ทั้งสามวิธีนี้มีประสิทธิภาพสูงกว่าแบบการทำนายการสุ่มอย่างมีนัยสำคัญ เป็นการแสดงให้เห็นความไปได้ในการประยุกต์ใช้หลักการทางด้านการประมวลผลสัญญานดิจิตัลเพื่อสร้าง Technical Indicator เพื่อมีประสิทธิภาพในกรารวิเคราะห์ตลาดหุ้น
]

#section("Abstract")[
  This project present
  #lorem(100)
]

// 1. บทนำ
#text(size: 20pt, weight: "bold")[1 บทนำ] \
#text(size: 18pt, weight: "bold")[1.1 ที่มาและความสำคัญ] \
#tab ในสถานการณ์ปัจจุบันโลกนั้นได้เจอกับการเปลี่ยนแปลงโดยการมีมาตราการจากนโยบายการเงินที่ผ่อนคลายเชิงระบบ (Quantative Easing) ของธนาคารกลางทั่วโลกทำให้เกิดการขยายตัวของปริมาณเงินในระบบหรือที่เรียกว่าเงินเฟ้อ โดยข้อมูลจาก Fedoral Reserve แสดงให้เห็นว่าเงิน M2 เพิ่มขึ้นทำมากมาย\
#tab ดังนั้นถ้าอยากให้ความสามารถในการจับจ่ายใช้สอยได้เท่าเดิมเราต้องเอาเงินของเราไปลงทุนเพื่อที่จะทำให้เงินของเรางอกเงย\
#tab โครงงานชิ้นนี้จะพัฒนาและวิเคราะห์โดยใช้ความรู้ทางด้าน Digital signal processing เพื่อสร้างตัวชีวัด (indicator) ที่สามารถตรวจวัดการเปลี่ยนแปลงของตลาด

#text(size: 18pt, weight: "bold")[1.2 วัตุประสงค์] \
1. พัฒนาระบบวิเคราะห์แน้วโน้มของตลาดหุ้น S&P 500 โดยใช้เทคนิค Moving Average Crossover และ statical model (ARIMA)
2. สร้าง Technical Indicator ที่มีความแม่นยำมากกว่าการเดาสุ่ม (50%) โดยต้องมากกว่าแบบมีนัยสำคัญ
3. วิเคราะห์คุณสมบัติของ stationrity ของข้อมูล S&P 500 index ด้วย ADF และ KPSS test

#text(size: 18pt, weight: "bold")[1.3 ขอบเขต] \
โครงงานนี้มีขอบเขตดังต่อไปนี้ :
1. ขอบเขตด้านปฏิบัติ
- ใช้ข้อมูลราคาปิดของกราฟรายวัน (daily) ของ S&P 500 index (^GSPC) จาก Yhaoo Finance
- ช่วงเวลา: 1 มกราคม 2015 ถึง 1 มกราคอม 2025 (เป็นระยะเวลา 10 ปี)
- จำนวนจุดข้อมูล: 2,516 obervations
- ประเภทข้อมูล: ราคาปิด (close price) ในรูปแบบ time series ที่สัญญานเป็น non-stationary

2. ขอบเขตด้านวิธีการ
- Technical analysis: ศึกษาแค่ Moving average indicator (SMA และ EMA)
- Optimization: Gird Search 100x100 (10,000 combination)
- Evalution: เน้นความแม่นยำ accuracy ไม่รวม profit/loss

3. ขอบเขตด้านการทดสอบ
- ทดสอบประสิทธิภาพด้วย rolling window prediction (one-step ahead)
- ใช้ entire dataset ไม่มีการแบ่ง train / test

// 2. ทฤษฎีทีนำเสนอ
#text(size: 20pt, weight: "bold")[2 ทฤษฏีที่นำเสนอ] \

#text(size: 18pt, weight: "bold")[2.1 ทฤษฏีที่เกี่ยวข้อง (Theoretical Background)] \
#text(size: 18pt, weight: "bold")[2.1.1 เส้นค่าเฉลี่ยนเคลื่อนที่ (Moving average)] \
*นิยาม* Moving average (MA) เป็นตัวกรองแบบ Finite Impulse Response (FIR) ที่ใช้ในทางด้าน Digital Signal Processing โดยตัว Moving Average (MA) ทำหน้าที่เป็น Low-pass Filter เพื่อลดความถี่สูงของสัญญานรบกวน (high-frequency noise) และทำให้เห็นแนวโน้มพื้นฐาน (underlying trend) ขออนุกรมเวลา \
โดยทำการเฉลี่ยค่าล่าสุดในหน้าต่างขนาด n \
$"MA"_(w)(t) = (1/w) sum_(i=t-w+1)^(t) x_(k)$\
โดยที่: \
- $"MA"_(w)(t)$ คือ ค่าเฉลี่ยเคลื่อนที่ (Moving average) ณ เวลาที่ด้วยขนาดหน้าต่าง $w$
- $w$ คือ ขนาดหน้าต่าง
- $x_(i)$ คือค่าที่เวลา $i$
- $t$ คือ เวลาปัจจุบัน

คุณสมบัติทางคณิตศาสตร์ (Mathematical Properties):\
1. Linear Time-Invarient(LTI) System: MA เป็นระบบ LTI ที่มีคุณสมบัติ causality และ stability
2. Phase Delay: ทำให้เกิด phase delay เท่ากับ $(omega-1)/2$ samples
3. Smoothing Factor: ระดับการปรับเรียบผกผันกับ $omega$
*Implement*: ใน rust ระบบใช้ module sma.rs ที่ implement โดยใช้ O(n) สำหรับการคำนวณ\

การใช้ MA ช่วยลดความผันผวนของสัญญานและช่วยให้มองเห็นแนวโน้มที่ชัดเจนยิ่งขึ้นแม้ว่าหุ้นนั้นจะแกว่งมากก็ตาม
\

#text(size: 20pt, weight: "bold")[2.1.2 เส้นค่าเฉลี่ยนเคลื่อนที่แบบเอ็กซ์โพเนนเชียล (Exponential moving average)] \
*นิยาม* Exponential Moving Average (EMA) เป็นตัวกรอกแบบ Infinite Impulse Response (IIR) ที่ให้น้ำหนักแบบ exponential decaying กับข้อมูลในอดีต ทำให้มีการตอบสนอง (response) ที่รวดเร็วกว่า MA\
$"EMA"_(w)(t) = x_(k) (S/(1+"Days")) + "EMA"_("t-1") (1- (S/(1+"Days")))$ \
โดยที่: \
- $alpha = S/(1 + "Days")$ = smoothing factor
- $S$ = smoothing constant (ใช้ค่า 2 ในโปรเจคนี้)
- $"Days"$ = จำนวน periods (windows)
- $x_(t)$ = ค่าปัจจุบัน
- $"EMA"_(t-1)$ = ค่า EMA ก่อนหน้า

คุณสมบัติทางศาสตร์ (Mathematical Properties):
1. Memory Effect: EMA มี infinite memory แต่ให้น้ำหนักลดแบบ exponential: $w_(k) = alpha(1-alpha)^(k)$
2. Effective WIndow: หน้าต่างทำให้เกิด phase delay เท่ากับ $2/alpha - 1$ periods
3. Lag Reduction: มี lag น้อยกว่า SMA เท่ากัย $(omega-1)/2$ periods
*Implementation*: ema.rs ใช้ interative approach ด้วย time complexity O(n) และ space complexity O(1)

จะเป็นสูตรที่ตอบสนองต่อการเปลี่ยนแปลงของข้อมูลได้เร็วกว่าจะได้รู้ว่า thread ของตลาดไปทางไหนได้เร็วกว่าแบบเคือ MA
// อธิบายด้วยว่าแต่ละตัวคืออะไร

#text(size: 20pt, weight: "bold")[2.1.3 Auto-Regressive Integrated Moving Average] \
*ARIMA* คือโมเดลสำหรับวิเคราะห์อนุกรมเวลา โดยรวมองค์ประกอบของการถดถอยอัตโนมัติ (Auto-Regressive) และค่าเฉลี่ยเคลื่อนที่ (Mean Average) เข้าด้วยกัน พร้อมกับการทำให้ข้อมูลเป็นสถิติก่อนด้วยการหาค่าต่าง (differencing) จำนวน d ครั้ง โมเดล ARIMA สามารถเขียนเป็น:  \
$Y_t = alpha + beta_(1}Y_(t-1) + beta_(2)Y_(t-2) + .. + beta_(p)Y_(t-p)epsilon_(t-1) + Phi_(1)epsilon_(t-1) + Phi_(2)epsilon_(t-2) + Phi_(q)epsilon_(t-q)$ \
เลือก parameter โดยการ
โดยที่:
- $Y_t$ = ค่าที่สังเกต ณ เวลา t (หลัง differencing)
- $c$ = ค่าคงที่ (drift term)
- $Phi_i$ = สัมประสิทธ์ Auto-Regressive(AR) ที่ lag i ($|Phi_i| < 1$ สำหรับ stationarity)
- $Theta_j$ = สัมประสิทธ์ Moving-average (MA) ที่ lag j ($|Theta_j|$ < 1 สำหรับ invertibility)
- $epsilon_t$ = white noise error term $~ N(0, sigma^2)$
- $L$ = lag Operator ($L^k dot Y_t = Y_(t-k)$)

การระบุ Parameters (model Identification):
1. Differencing Order (d):
  - ใช้ Augmented Dickey-Fuller (ADF) test
  - KPSS test สำหรับ trend stationarity
2. AR Order (p) ผ่าน PACF:
  - ใช้ OLS regression
  - Cut-off criterion: $|"PACF"(K)| < 1.96/(sqrt(n))$
3. MA Order (q) ผ่าน ACF:
  - ใช้ FFT-base Compute สำหรับงาน efficiency
  - Autcorrelation function ที่ lag k
// อธิบานว่าแต่ละตัวคืออะไรขอแบบละเอียดเลยอันนี้

#text(size: 20pt, weight: "bold")[2.1.4 การแปลงฟูเรียร์แบบเร็ว (Fast Fourier Transform)] \
เป็น algorithm ที่ optimal ที่สุดในเวลานี้สำหรับการคำนวณ Discrete Fourier Transform (DFT):
$X[k] = sum_(n=0)^(N-1)(x[n] * e^((-j 2 pi k n )/ N))$
การนำมาประยุกต์:
1. Spectral Density Estimation
2. ACF via Winer-Khinchin Theorem
3. Periodictiy Detection

// 2.2 การประเมินประสิทธิภาพของระบบ
#text(size: 20pt, weight: "bold")[2.2 การประเมินประสิทธิภาพของระบบ] \

1. Accuracy (ความถูกต้องโดยรวม): $"Accuracy" = ("TP" + "TN")/ "Total"$
2. Precision (ความแม่นยำ):\ $"Precision" = "TP"/("TP" + "FP")$
3. Recall (ความไว):\ $"Recall" = "TP"/("TP" + "FN")$
// 2.3 วิธีการแก้ปัญหา

#text(size: 20pt, weight: "bold")[2.3 วิธีการแก้ปัญหา] \
#text(size: 20pt, weight: "bold")[2.3.1 โครงสร้างระบบ (System Architecture)]\
```bash
src/module/
├── data
├── eval.rs
├── indicator
├── model
├── mod.rs
├── plot
├── util
└── workflow.rs
```

Design Principles:
1. Vibe Coding

#text(size: 20pt, weight: "bold")[2.3.1 Algorithm Implementation Details]\
EMA Crossover Strategy:
- Fast Ema (period สั้น) ตัดบน Slow Ema (period ยาว) = Buy Signal
- Zero-padding สำหรับ boundary conditions
- Time Complexity: O(n) per prediction

ARIMA (Auto-Regressive integrate moving average):\
1. Stationnary testing -> ADF/KPSS test
2. Parameter Selection -> PADF/ACF analysis
3. Model fitting -> CSSoptimization
4. Rolling Prediction -> One-step ahead forecasts
5. Performance Evalution -> Direactional accuracy
// 2.3.1 การออกแบบระบบโดยรวม พร้อมแนบรูป
// 2.3.2 การพัฒนา software
// etc...
// 3 ผลการทดลอง
#text(size: 20pt, weight: "bold")[3 ผลการทดลอง] \
การทดสอบใช้ข้อมูลของหุ้น S&P 500 Index (^GSPC) จาก Yahoo Finanace ระหว่างวันที่ 1 มกราคม 2015 ถึง 1 มกราคม 2025 ทั้งหมดคือ 2,516 จุดรายวัน (daily) โดยใช้ตัวแปรคือราคาปิดของวันนั้น \

ข้อมูลทั้งหมดที่เป็นความต่างของข้อมูลวันนี้และเมื่อวาน (logs return) จะถูกนำไปใช้ใน ARIMA เพื่อการพยากรณ์ว่าจะขึ้นหรือลงวันวันพรุ่งนี้ที่ (T+1) โดยต้องนำไปทดสอบเพื่อหา parameter คือ ค่า d ว่าเท่ากับเท่าไร
ถ้าเป็น stationary จะให้ค่า d = 0 ถ้าไม่ใช้ stationary จะให้ค่า d = 1

#align(center)[
  #table(
    columns: (auto, auto, auto, auto),
    inset: 10pt,
    align: (left, center, center, left),
    [*การทดสอบ*], [*ค่าสถิติ*], [*ค่าวิกฤต (1%)*], [*ผลการทดสอบ*],
    [ADF Test (Diff)], [$t = -41.160$], [$-3.460$], [Stationary],
    [KPSS Test (Diff)], [$0.175$], [$0.739$], [Level-stationary],
  )
]

สรุป:ผลการทดสอบ ADF แสดงค่า t-statistic ที่ต่ำกว่าค่าวิกฤตอย่างมีนัยสำคัญ ($p < 0.01$) ทำให้ปฏิเสธสมมติฐานหลักของ unit root ได้ ขณะที่ KPSS test ไม่สามารถปฏิเสธสมมติฐานหลักของ stationarity ได้ แสดงว่าข้อมูลมีคุณสมบัติ stationary เหมาะสมสำหรับการใช้งาน

จากผลดังตารางเราจะให้ค่า d = 0 เป็น stationary จะเรียกว่า ARMA(p,q)

#text(size: 17pt, weight: "bold")[ การเลือกพารามิเตอร์ของโมเดล] \
การวิเคราะห์ Partial Autocorrelation Function (PACF) ด้วยวิธี Levinson-Durbin และ OLS regression ให้ผลสอดคล้องกันที่ $p = 1$ โดยใช้เกณฑ์ cutoff ที่ระดับความเชื่อมั่น 95% ($±1.96/ sqrt(n) = ±0.0391$)

#text(size: 17pt, weight: "bold")[การหาค่า MA Order (q) ด้วย ACF] \
การวิเคราะห์ Autocorrelation Function ด้วย FFT-based algorithm แสดงผลดังนี้:
- First-drop criterion: $q = 2$
- Largest significant lag: $q = 10$
จะพิจารณาเลือกใช้ frist drop

#text(size: 17pt, weight: "bold")[3.4.1 Grid Search Optimization] \
จะทำการค้นหาค่าที่ดีที่สุด (accuracy) สูงสุดด้วยการทดสอบ 100x100ของ ema/sma period
ได้ผลดังนี้
#align(center)[
  #table(
    columns: (auto, auto, auto, auto),
    inset: 10pt,
    align: (left, center, center, center),
    [*Strategy*], [*Parameters*], [*Accuracy*], [*Coverage*],
    [Best], [EMA(11) × SMA(99)], [53.45%], [96.10%],
  )
]

จากการทดลองแสดงให้เห็นว่า\
โมเดล EMA(11) x SMA(99) ให้ความแม่นยำสูงสุดที่ 53.45% ด้วย precision 54.90% และ recall 77.24% ตามมาด้วย EMA(11) x EMA(99) ที่ให้ความแม่นยำ 52.83% (precision 54.36%, recall 78.77%) และ ARIMA(1,0,2) ให้ความแม่นยำ 52.19% (precision 53.80%, recall 78.16%)

การวิเคราะห์รายละเอียดของ Best Model (EMA(11)×SMA(99)) จากจำนวนข้อมูลทั้งหมด 2,417 จุด แบ่งเป็น 4 กลุ่มดังนี้ True Positive (ทำนายขึ้นและขึ้นจริง) 1,008 ครั้ง คิดเป็น 41.7% ของทั้งหมด True Negative (ทำนายลงและลงจริง) 284 ครั้ง คิดเป็น 11.8% False Positive (ทำนายขึ้นแต่ลงจริง) 297 ครั้ง คิดเป็น 12.3% และ False Negative (ทำนายลงแต่ขึ้นจริง) 828 ครั้ง คิดเป็น 34.3%

โมเดล ARIMA(1,0,2) ที่ได้จากการประมาณค่าด้วย Conditional Sum of Squares ให้สมการพยากรณ์ดังนี้ $Y_t = 9.020 + 0.998 Y_(t-1) + epsilon_t - 0.043 epsilon_(t-1) + 0.016 epsilon_(t-2)$ ค่าสัมประสิทธิ์ AR ที่ 0.998 โดยความถูกต้องโดยรวมอยู่ที่ 52.19% และ ความแม่นยำอยู่ที่ 53.80%

// 4 การวิเคราะห์และสรุปผล
#text(size: 20pt, weight: "bold")[4 การวิเคราะห์และสรุปผล] \
1. ประสิทธิภาพของ technical indicator: ด้วยใช้วิธี EMA crossover จะได้ประสิทธิภาพที่ EMA(11) x EMA(99) ได้ผลลัพธ์ที่ดีที่สุดโดยวัดจากความแม่นยำ accuracy คือ 53.35% และ precision ที่ 54.90%

2. การเลือกใช้ period ที่เหมาะสมสำหรับการทำ indicator อาจจะปรับให้มากกว่านี้เช่น EMA(11) โดยเลข 11 ยังเป็นเลขที่มีนัยสำคัญมากในการวิเคราะห์โดยการใช้ indicator ครั้งที่

3. ข้อจำกัดของ ARIMA: ตลาดหุ้นเป็น non linear และ deterministic ทำให้ ARIMA ไม่สามารถประมาณค่าได้แม่นยำมาก โดยในการทดลองนี้ใกล้เคียงกับ moving average crossover

โครงงานนี้แสดงให้เห็นการประยุกต์โดยใช้เทคนิคทาง digital signal processing เพื่อสร้าง threading indicator ที่มีประสิทธิภาพเหนือกว่าการสุ่มแบบโยนเหรียญเล็กน้อย

// 5 กิตติกรรมประกาศ
#text(size: 20pt, weight: "bold")[5 กิตติกรรมประกาศ] \


// 6 เอกสารอ้างอิง
#text(size: 20pt, weight: "bold")[6 เอกสารอ้างอิง] \

[1] Algorithmic-Oriented Digital Signal Processing for Computer Engineers 01204496, “บท 5: การประมวลผลสัญญาณปรับได้ (Adaptive Signal Processing),” lecture notes, [in Thai].

[2] Anthropic, “Claude AI [Large language model],” Sep. 2025. [Online]. Available: #link("https://www.anthropic.com/claude")

[3] P. S. R. Diniz, Signal Processing and Machine Learning Theory. Academic Press, 2024.

[4] G. Gundersen, “Returns and Log Returns,” Sep. 2022. [Online]. Available: #link("https://gregorygundersen.com/blog/2022/02/06/log-returns/")

[5] OpenAI, “ChatGPT [Large language model],” Sep. 2025. [Online]. Available: #link("https://chatgpt.com")

[6] S. Prabhakaran, “Time Series Analysis in Python – A Comprehensive Guide with Examples,” MachineLearningPlus (ML+), Feb. 13, 2019. [Online]. Available: #link("https://www.machinelearningplus.com/time-series/time-series-analysis-python/")

[7] The Rust Project Developers, The Rustdoc Book. [Online]. Available: #link("https://doc.rust-lang.org/rustdoc/")

[8] Machine Learning Plus, “ARIMA Model — Time Series Forecasting in Python.” [Online]. Available: https://www.machinelearningplus.com/time-series/arima-model-time-series-forecasting-python/. Accessed: Sep, 2025.
