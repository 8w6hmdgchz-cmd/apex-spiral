
| 科研场景 | 推荐模型 | 关键参数 | 输出指标 |
|---------|---------|---------|---------|
| RCT疗效(二分类) | GLM+Bernoulli+Logit | family=Binomial() | OR, 95%CI, p |
| RCT疗效(连续) | OLS/GLM+Gaussian | family=Gaussian() | MD, 95%CI, p |
| 生存分析(OS/PFS) | CoxPH | PHReg() | HR, 95%CI, p |
| Meta二分类 | Mantel-Haenszel/statsmodel | GLM+随机效应 | OR/RR, I² |
| 诊断试验 | Logistic | GLM+Binomial | AUC, Sen, Spe |
| 重复测量 | GEE | cov_struct=Exchangeable | OR/MD, QIC |
| 倾向性评分 | Logistic→WLS | PS→IPTW/Matching | ATE, ATT |
