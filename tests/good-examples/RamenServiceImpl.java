package com.entrust.ecs.ramen;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.lang.invoke.MethodHandles;
import java.lang.reflect.Type;
import java.math.BigInteger;
import java.net.MalformedURLException;
import java.net.URL;
import java.security.cert.CertificateEncodingException;
import java.security.cert.CertificateException;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Date;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.Vector;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReentrantLock;
import java.util.stream.Collectors;

import javax.annotation.PostConstruct;
import javax.security.auth.x500.X500Principal;
import javax.servlet.ServletException;
import javax.validation.Valid;
import javax.ws.rs.core.Response.Status;

import org.apache.commons.beanutils.BeanUtils;
import org.apache.commons.codec.binary.Base64;
import org.apache.commons.collections.EnumerationUtils;
import org.apache.commons.io.IOUtils;
import org.apache.commons.lang3.ArrayUtils;
import org.apache.commons.lang3.StringUtils;
import org.apache.commons.lang3.exception.ExceptionUtils;
import org.apache.commons.lang3.time.DateUtils;
import org.apache.commons.lang3.time.StopWatch;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.slf4j.MDC;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.core.env.Environment;
import org.springframework.http.MediaType;
import org.springframework.stereotype.Component;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import com.entrust.adminservices.toolkit.AtkVersion;
import com.entrust.eaf.utility.CertificateUtil;
import com.entrust.ecs.EcsObjectIDs;
import com.entrust.ecs.EntrustRA.SignAlgType;
import com.entrust.ecs.EntrustRA.SslCertType;
import com.entrust.ecs.businessentity.TrustChainCertificate;
import com.entrust.ecs.contract.beans.Rfc6844Result;
import com.entrust.ecs.contract.service.CaaService;
import com.entrust.ecs.contract.service.CertificateDataService;
import com.entrust.ecs.contract.service.CertificateDataService2;
import com.entrust.ecs.contract.service.CertificatePickupService;
import com.entrust.ecs.contract.service.CertificateService2;
import com.entrust.ecs.contract.service.CmsClientService;
import com.entrust.ecs.contract.service.CmsService;
import com.entrust.ecs.contract.service.CustomerService;
import com.entrust.ecs.contract.service.HealthStatus;
import com.entrust.ecs.contract.service.OrphanedCertEvent;
import com.entrust.ecs.entity.OrphanedCert;
import com.entrust.ecs.entityadaptor.LintingStatus;
import com.entrust.ecs.entityadaptor.RAInfo2;
import com.entrust.ecs.ra.client.CreateRequest;
import com.entrust.ecs.ra.client.RACertificateResponse;
import com.entrust.ecs.ra.client.RAFormRequest;
import com.entrust.ecs.ra.client.RARequestParam;
import com.entrust.ecs.ra.client.RAResponse;
import com.entrust.ecs.ra.client.RARevokeRequest;
import com.entrust.ecs.ra.client.RARevokeResponse;
import com.entrust.ecs.ra.client.RevokeRequest;
import com.entrust.ecs.ra.client.VangSyncRequest;
import com.entrust.ecs.ramen.ca.CaCreateRequest;
import com.entrust.ecs.ramen.ca.CaService;
import com.entrust.ecs.ramen.ca.CaServicesConfig;
import com.entrust.ecs.ramen.ca.cagw.CagwCaService;
import com.entrust.ecs.ramen.ca.cagw.CagwConfig;
import com.entrust.ecs.ramen.ca.sml.SmlCaService;
import com.entrust.ecs.ramen.ca.sml.SmlConfig;
import com.entrust.ecs.ramen.ca.tk.CaPoolConfig;
import com.entrust.ecs.ramen.ca.tk.CaServiceConfig;
import com.entrust.ecs.ramen.ca.tk.ToolkitCaConfig;
import com.entrust.ecs.ramen.ca.tk.ToolkitCaPoolService;
import com.entrust.ecs.ramen.ca.tk.ToolkitCaService;
import com.entrust.ecs.ramen.keygen.KeyGenerator;
import com.entrust.ecs.ramen.preprocess.AddBimiExtension;
import com.entrust.ecs.ramen.preprocess.AddCnToSan;
import com.entrust.ecs.ramen.preprocess.AddQcPsd2Info;
import com.entrust.ecs.ramen.preprocess.AddQcStatementsExtension;
import com.entrust.ecs.ramen.preprocess.AddCsr;
import com.entrust.ecs.ramen.preprocess.CreateReqModifier;
import com.entrust.ecs.ramen.preprocess.DedupeSanStringAndCleanIPs;
import com.entrust.ecs.ramen.preprocess.OvCtOverride;
import com.entrust.ecs.ramen.preprocess.SetExpiryTime;
import com.entrust.ecs.ramen.preprocess.TruncateExpiryDays;
import com.entrust.ecs.service.EcsFault;
import com.entrust.ecs.service.caa.GsonUtil;
import com.entrust.ecs.service.cert.CtUtil;
import com.entrust.ecs.service.cert.EcsCRLReason;
import com.entrust.ecs.service.cert.SanString;
import com.entrust.ecs.service.cert.SanStringType;
import com.entrust.ecs.service.cert.ValidationResult;
import com.entrust.ecs.service.lint.LintingManager;
import com.entrust.ecs.service.lint.ZLinterConfig;
import com.entrust.ecs.validation.Message;
import com.entrust.ecs.vang.FeederApiClient;
import com.entrust.ecs.vang.FeederResponse;
import com.entrust.ecs.vang.VangRevokeRequest;
import com.entrust.ecs.vang.VangRevokeRequest.AkiSerialNumber;
import com.entrust.toolkit.security.provider.Initializer;
import com.entrust.toolkit.util.SecureStringBuffer;
import com.entrust.toolkit.x509.extensions.SignedCertificateTimestampList;
import com.google.common.base.Preconditions;
import com.google.common.base.Stopwatch;
import com.google.common.base.Strings;
import com.google.common.cache.Cache;
import com.google.common.cache.CacheBuilder;
import com.google.common.io.BaseEncoding;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.reflect.TypeToken;

import iaik.asn1.ObjectID;
import iaik.asn1.structures.Name;
import iaik.pkcs.PKCSException;
import iaik.pkcs.pkcs12.CertificateBag;
import iaik.pkcs.pkcs12.KeyBag;
import iaik.pkcs.pkcs12.PKCS12;
import iaik.utils.RFC2253NameParser;
import iaik.utils.RFC2253NameParserException;
import iaik.x509.V3Extension;
import iaik.x509.X509ExtensionException;
import iaik.x509.X509Extensions;
import io.micrometer.core.instrument.Metrics;
import io.micrometer.core.instrument.Timer;

public class RamenServiceImpl {

  public static final Logger logger = LoggerFactory.getLogger(MethodHandles.lookup().lookupClass());

  public static final String MDC_REQUEST_ID = "requestId";

  private static final String CLIENT_TYPE = "RAMEN";
  private Environment env;

  private String caServiceDefault = null;
  
  private boolean skipValidation = false;

  private int duplicateRequestTimeInterval = 15;
  private static final int CACHE_LOCK_ACQUIRE_TIMEOUT = 15;
  private String lintingServiceUrl = null;

  private int lintingServiceConnectionManagerTimeout = 1000;
  private int lintingServiceConnectionTimeout = 1000;
  private int lintingServiceSocketTimeout = 100;
  
  /**
   * Key generation params
   * (Some cert types make requests without CSRs, requiring us to generate the keys and return a pkcs12)
   */
  private boolean enableKeyGen = false;
  private KeyGenerator keyGenerator = null;

  private String keyAlgorithm = null;
  private int keySize = 2048;
  private int maxQueueSize = KeyGenerator.DEFAULT_QUEUE_SIZE;
  private int interval = KeyGenerator.DEFAULT_INTERVAL;
  private int delay = KeyGenerator.DEFAULT_DELAY;
  private int logInterval = 15;

  /**
   * Cache of userDN for recent Create requests
   */
  private Cache<String, Boolean> createReqCache = null;
  private Lock createReqCacheLock = new ReentrantLock();

  /**
   * Cache of userDN and serial number for recent Revoke requests
   */
  private Cache<String, String> revokeReqCache = null;
  private Lock revokeReqCacheLock = new ReentrantLock();

  private CaService defaultCaService = null;

  private CertificateDataService certificateDataService = null;
  private CertificateDataService2 certificateDataService2 = null;
  private CertificatePickupService certificatePickupService = null;
  private CmsClientService cmsClientService = null;
  private CmsService cmsService = null;
  private CertificateService2 certificateService2 = null;
  private CaaService caaService = null;
  private CustomerService customerService = null;

  // FeederApiClient (i.e. VANG integration may be disabled via feeder.api.enabled=false
  FeederApiClient feederApiClient = null;

  private boolean ignoreVangErrors = false;

  private Map<String, CaService> caServices = new HashMap<>();

  private CaServicesConfig caServicesConfig = null;

  Timer createTimer = null;
  Timer caaTimer = null;
  Timer validateTimer = null;
  Timer ctTimer = null;
  Timer lintTimer = null;
  Timer syncTimer = null;
  
  private static ThreadLocal<CreateState> createState = ThreadLocal.withInitial(CreateState::new);

  public RamenServiceImpl() {

    logger.info(com.entrust.toolkit.util.Version.VERSION);
    logger.info( AtkVersion.VERSION );
    logger.info( com.entrust.toolkit.util.Version.getJavaRuntimeInfo() );
    logger.info( com.entrust.toolkit.util.Version.getJavaVirtualMachineInfo() );
    logger.info( com.entrust.toolkit.util.Version.getOperatingSystemInfo() );

    Stopwatch stopwatch = Stopwatch.createStarted();
    Initializer.getInstance().setProviders(Initializer.MODE_NORMAL);
    logger.info( "Initialized Entrust JCE provider in " + stopwatch.elapsed(TimeUnit.MILLISECONDS) + " ms" );

    logger.info("duplicateRequestTimeInterval=" + duplicateRequestTimeInterval);

    EcsObjectIDs.registerEVObjectIds();
  }

  public String info() {

    String infoUrl =
        "http://localhost:" + env.getProperty("management.server.port") + env.getProperty("management.endpoints.web.base-path") +"/info";
    try {
      URL url = new URL(infoUrl);
      return IOUtils.toString(url);
    } catch (MalformedURLException e) {
      String errorMsg = "Malformed info URL: " + infoUrl;
      logger.error(errorMsg);
      return errorMsg;
    } catch (IOException e) {
      String errorMsg = "Failed to read: " + infoUrl;
      logger.error(errorMsg);
      return errorMsg;
    }
  }

  public String toString() {
    return "RamenServiceImpl [skipValidation=" + skipValidation + ", duplicateRequestTimeInterval="
        + duplicateRequestTimeInterval + ", ignoreVangErrors=" + ignoreVangErrors + ", keyGeneration="
        + (enableKeyGen ? "enabled:" + keyGenerator.toString() : "disabled") + "]";
  }

  public String health() {

    try {
      boolean allUp = true;
      boolean allDown = true;
      for (Map.Entry<String, CaService> caServiceEntry : caServices.entrySet()) {
        switch ( caServiceEntry.getValue().health()) {
          case UP:
            allDown = false;
            break;
          case DOWN:
            allUp = false;
            break;
          case DEGRADED:
            allUp = false;
            allDown = false;
            break;
          default:
            // This should not happen
            return HealthStatus.DOWN.toString();
        }
      }
      if (allUp) {
        return HealthStatus.UP.toString();
      } else if (allDown) {
        return HealthStatus.DOWN.toString();
      } else {
        return HealthStatus.DEGRADED.toString();
      }
    } catch (Exception e) {
      logger.error("Failed to get health: " + e.getMessage(), e);
      return HealthStatus.DOWN.toString();
    }
  }

  public String raCertificate(RAFormRequest raFormReq) throws Exception {
    return raCertificate(raFormReq.getCa(), raFormReq.getRequest(), raFormReq.getUserDN(), raFormReq.getTrackingID(),
        raFormReq.getAccountNumber(), raFormReq.getOrderID(), raFormReq.getCertType(), raFormReq.getSslCertRule(),
        raFormReq.getUserCertExpiryDays(), raFormReq.getDomainID(), raFormReq.getUserPKCS10(),
        raFormReq.getSubjectAltName(), raFormReq.getCtLog(), raFormReq.getP12password(), raFormReq.getUserAgent(),
        raFormReq.getIncludeClass2KeyHistory(), raFormReq.getCertificateSerialNumber(), raFormReq.getRevocationReason(),
        raFormReq.getRevocationText(), raFormReq.getLastGoodDate(), raFormReq.getIssueCRL());
  }

  public String raCertificate(String ca, String request, String userDN, String trackingID, String accountNumber, String orderID,
      String certType, String sslCertRule, String userCertExpiryDays, String domainID, String userPKCS10,
      String subjectAltName, String ctLog, String p12password, String userAgent, String includeClass2KeyHistory,
      String certificateSerialNumber, String revocationReason, String revocationText, String lastGoodDate,
      String issueCRL) throws Exception {
    try {
      if (RARequestParam.REQ_PARAM_REQUEST_VALUE_CREATE.equals(request)) {
        Preconditions.checkArgument(!Strings.isNullOrEmpty(userCertExpiryDays), "userCertExpiryDays cannot be null or empty");
        CreateRequest createReq = new CreateRequest(userDN, trackingID, accountNumber, orderID, certType, sslCertRule,
            Integer.parseInt(userCertExpiryDays), (domainID == null ? null : Integer.parseInt(domainID)), userPKCS10,
            subjectAltName, ctLog == null ? null : Boolean.valueOf(ctLog), null, null, null);
        RACertificateResponse createRes = createCertificate(caServiceDefault, createReq);
        return new GsonBuilder().create().toJson(createRes);
      } else if (RARequestParam.REQ_PARAM_REQUEST_VALUE_REVOKE.equals(request)) {
        RevokeRequest revokeReq = new RevokeRequest(userDN, Integer.parseInt(trackingID), certificateSerialNumber,
            lastGoodDate == null ? null : RARevokeRequest.parseLastGootDate(lastGoodDate),
            Integer.parseInt(revocationReason), revocationText, Boolean.parseBoolean(issueCRL));
        RARevokeResponse revokeRes = revokeCertificate(caServiceDefault, revokeReq);
        revokeRes.setRequestId(trackingID);
        return new GsonBuilder().create().toJson(revokeRes);
      } else {
        throw new IllegalArgumentException("Unsupported \"request\" parameter: " + request);
      }
    } catch (Exception e) {
      String errMsg =
          "Unexpected exception: " + e.getMessage() + "\n" + "ca=" + ca + ", request=" + request + ", userDN=" + userDN
              + ", trackingID=" + trackingID + ", accountNumber=" + accountNumber + ", orderID=" + orderID
              + ", certType=" + certType + ", sslCertRule=" + sslCertRule + ", userCertExpiryDays=" + userCertExpiryDays
              + ", domainID=" + domainID + ", userPKCS10=" + userPKCS10 + ", subjectAltName=" + subjectAltName
              + ", ctLog=" + ctLog + ", userAgent=" + userAgent + ", includeClass2KeyHistory=" + includeClass2KeyHistory
              + ", certificateSerialNumber=" + certificateSerialNumber + ", revocationReason=" + revocationReason
              + ", revocationText=" + revocationText + ", lastGoodDate=" + lastGoodDate + ", issueCRL=" + issueCRL;
      logger.info(errMsg, e);
      throw new Exception(errMsg);
    }
  }

  public void init() throws Exception {

    createReqCache = CacheBuilder.newBuilder().expireAfterWrite(duplicateRequestTimeInterval, TimeUnit.SECONDS).build();
    revokeReqCache = CacheBuilder.newBuilder().expireAfterWrite(duplicateRequestTimeInterval, TimeUnit.SECONDS).build();

    initCaServices();

    // Default is for "legacy" mode where a CA service is not specified in the path
    logger.info("Using default CA service: {}", caServiceDefault);
    defaultCaService = caServices.get(caServiceDefault);

    if (defaultCaService == null) {
      String error = String.format("Default CA service \"%s\" is not available", caServiceDefault);
      logger.error(error);
      throw new ServletException(error);
    }

    if (skipValidation) {
      logger.warn("******************************************");
      logger.warn("***** DEVELOPMENT CONFIGURATION **********");
      logger.warn("******************************************");
      logger.warn("***** SKIPPING ALL VALIDATION ************");
      logger.warn("******************************************");
      logger.warn("***** MUST NOT BE USED IN PRODUCTION *****");
      logger.warn("******************************************");
    }

    if (ignoreVangErrors) {
      logger.warn("*****************************************");
      logger.warn("***** VANG MIGRATION TEST MODE **********");
      logger.warn("******* IGNORING VANG ERRORS ************");
      logger.warn("*****************************************");
    }

    createTimer = createReqTimer("total");
    caaTimer = createReqTimer("caa");
    validateTimer = createReqTimer("validate");
    ctTimer = createReqTimer("ct");
    lintTimer = createReqTimer("lint");
    syncTimer = createReqTimer("sync");

    if (enableKeyGen) {
      keyGenerator = new KeyGenerator(keyAlgorithm, keySize, maxQueueSize, interval, delay, logInterval);
    }
    
    logger.info(this.toString());

  }

  public RACertificateResponse createCertificate() {

    Stopwatch stopwatch = Stopwatch.createStarted();
    Timer.Sample createSample = Timer.start(Metrics.globalRegistry);

    try {
      logger.debug(new GsonBuilder().create().toJson(createReq));

      // "nullify" empty accountNumber and orderID
      createReq.setAccountNumber(StringUtils.stripToNull(createReq.getAccountNumber()));
      createReq.setOrderID(StringUtils.stripToNull(createReq.getOrderID()));

      CaCreateRequest caCreateReq = new CaCreateRequest(createReq, skipValidation);
      
      String requestId = null;
      if (caCreateReq.getRequestId() == null) {
        requestId = createReq.toHashString();
      } else {
        requestId = caCreateReq.getRequestId();
      }
      setMdcRequestId(requestId);
      
      // ECSPR-32822: Some requests might have 'UserDN=' prefix that needs to be sanitized
      caCreateReq.getCreateReq().setUserDN(CertificateUtil.sanitizeDN(caCreateReq.getCreateReq().getUserDN()));
      checkDuplicateCreateRequest(createReq);

      try {

        CaService requestedCaService = getRequestedCaService(ca);

        SslCertType sslCertType = SslCertType.STD;
        if (requestedCaService.isAffirmTrust()) {
          createReq.setSslCertRule(sslCertType.toString());
        } else {
          sslCertType = sslCertTypeFromRule(requestId, createReq.getSslCertRule());
        }

        runProcessors(
            // Truncate expiry days, if necessary
            new TruncateExpiryDays(caCreateReq, certificateDataService),
            // Add CN to SAN, if necessary
            new AddCnToSan(caCreateReq),
            // De-dupe sanString and convert IP Addresses encoded as dNSName to iPAddresses
            new DedupeSanStringAndCleanIPs(caCreateReq)
            );

        // If not skipping all validation, check isCtEnabled for this RA from the DB
        // CT mode is disabled when skipping all validation
        RAInfo2 raInfo = certificateDataService.getRAInfoByName(requestedCaService.getCaName());
        caCreateReq.setCtEnabled(raInfo.isCtEnabled());
        logger.info("CT logging is " + (caCreateReq.isCtEnabled() ? "enabled" : "disabled")
              + " for this RA (" + raInfo.getName() + ")");
        CaaCheckResult caaCheckResult = processCaaCheckResult(raInfo, caCreateReq.getCreateReq().getSubjectAltName(),
            caCreateReq.getCreateReq().getTrackingID(), caCreateReq.getCreateReq().getAccountNumber(),
            caCreateReq.getCreateReq().getOrderID());

        if (!requestedCaService.isAffirmTrust()) {
          runProcessors(
              // Set expiry
              new SetExpiryTime(caCreateReq, certificateDataService2),
              new OvCtOverride(caCreateReq, cmsService));
        } else {
          // We need to still set the gmtKeyExpiryDate that would have been populated inside SetExpiryTime preprocessor
          caCreateReq
              .setGmtKeyExpiryDate(DateUtils.addDays(new Date(), caCreateReq.getCreateReq().getUserCertExpiryDays()));
        }
        
        logger.info((skipValidation ? "***** SKIPPING SKIPPING VALIDATION OF PARAMETERS:" : "Ready to validate: ")
            + createReq + ", gmtKeyExpiryDate=\'" + caCreateReq.getGmtKeyExpiryDate() + "\', logToCt=" + caCreateReq.isCtEnabled());

        validateCreateReq(requestId, createReq.getOrderID(), createReq.getAccountNumber(),
            createReq.getDomainID() == null ? 0 : createReq.getDomainID(), createReq.getTrackingID(), createReq.getUserPKCS10(), 
            caCreateReq.getCreateReq().getUserDN(), caCreateReq.getCreateReq().getSubjectAltName(), sslCertType,
            caCreateReq.getGmtKeyExpiryDate(), requestedCaService.isAffirmTrust());

        long elapsed1 = stopwatch.elapsed(TimeUnit.MILLISECONDS);
        RamenStats.log("RamenServiceImpl.createCertificate().validateCreateReq", elapsed1);

        String b64FinalCert = null;
        runProcessors(
            // Add type specific extension(s), if necessary
            new AddBimiExtension(caCreateReq),
            new AddQcStatementsExtension(caCreateReq),
            new AddQcPsd2Info(caCreateReq, cmsClientService),
            new AddCsr(caCreateReq, keyGenerator));

        // Add any custom extensions to final cert
        X509Extensions finalCertExts = new X509Extensions();
        addCustomExtensions(finalCertExts, caCreateReq.getCustomExtensions());

        CertificateFactory cf = CertificateFactory.getInstance("X.509");
        X509Certificate x509PreCert = null;
        X509Certificate x509Cert = null;

        if (caCreateReq.isCtEnabled()) {

          X509Extensions preCertExts = new X509Extensions();
          addCustomExtensions(preCertExts, caCreateReq.getCustomExtensions());

          String b64PreCert =
              requestedCaService.createCertificate(requestId, createReq.getUserDN(), createReq.getCertType(),
                  createReq.getUserPKCS10(), createReq.getSubjectAltName(), caCreateReq.getGmtKeyExpiryDate(),
                  preCertExts, true);
          logger.info("PreCert: " + b64PreCert);
          
          long elapsed2 = stopwatch.elapsed(TimeUnit.MILLISECONDS);
          RamenStats.log("RamenServiceImpl.createCertificate().preCert", elapsed2-elapsed1);
          // Lint if linting is enabled.
          Timer.Sample lintSample = Timer.start(Metrics.globalRegistry);
          try {
            if (raInfo.getCertLinting() != LintingStatus.Disabled) {
              logger.info(CLIENT_TYPE + ": " + raInfo.getName() + " calling linting service for CtPreCert linting");
              LintingManager lintingManager = getLintingManager(raInfo);
              if (! lintingManager.lint(LintingManager.ERROR_CERTIFICATE_FAILED_LINTING, raInfo.getCertLinting(), b64PreCert, requestId, createReq.getUserDN())) {
                throw new RaException(Status.BAD_REQUEST, LintingManager.ERROR_PRE_CERTIFICATE_FAILED_LINTING);
              }
            }
          } finally {
            lintSample.stop(lintTimer);
          }

          x509PreCert = (X509Certificate) cf.generateCertificate(
              new ByteArrayInputStream(BaseEncoding.base64().decode(b64PreCert)));
          
          // Sync pre-cert with VANG
          _syncCertWithVang(b64PreCert, ca, x509PreCert.getSerialNumber().toString(16));
          createState.get().setPrecertSynced(true);

          RamenAudit.logIssuedPreCert(
              ca, x509PreCert.getSubjectDN().getName(), x509PreCert.getSerialNumber().toString(16), b64PreCert);

          elapsed1 = elapsed2;

          // Get SCTs from logs
          Timer.Sample ctSample = Timer.start(Metrics.globalRegistry);

          List<String> jsonScts = null;
          try {
            jsonScts =
                certificateDataService.ctLogPreCert(createReq.getCertType(), b64PreCert, createReq.getSslCertRule());
          } finally {
            ctSample.stop(ctTimer);
          }
          elapsed2 = stopwatch.elapsed(TimeUnit.MILLISECONDS);
          RamenStats.log("RamenServiceImpl.createCertificate().ctLog", elapsed2-elapsed1);
          elapsed1 = elapsed2;

          // Build SCT extension
          SignedCertificateTimestampList sctList = CtUtil.parseJsonScts(jsonScts);

          b64FinalCert = requestedCaService.createCertificateWithSct(requestId, createReq.getCertType(),
              createReq.getUserPKCS10(), b64PreCert, sctList, finalCertExts);

          logger.info("Final Cert: " + b64FinalCert);
          elapsed2 = stopwatch.elapsed(TimeUnit.MILLISECONDS);
          RamenStats.log("RamenServiceImpl.createCertificate().finalCert", elapsed2-elapsed1);
          elapsed1 = elapsed2;

          boolean sctsVerified = certificateDataService.ctVerifyEmbeddedSct(createReq.getCertType(), b64FinalCert);
          elapsed2 = stopwatch.elapsed(TimeUnit.MILLISECONDS);
          RamenStats.log("RamenServiceImpl.createCertificate().ctVerifyEmbeddedSct", elapsed2-elapsed1);
          elapsed1 = elapsed2;
          if (!sctsVerified) {
            String msg = "Failed to verify embedded SCTs for cert: " + b64FinalCert;
            logger.error(msg);
            throw new RaException(Status.INTERNAL_SERVER_ERROR, msg);
          } else {
            logger.info("Verified embedded SCTs");
          }

          x509Cert = (X509Certificate) cf.generateCertificate(
              new ByteArrayInputStream(BaseEncoding.base64().decode(b64FinalCert)));
          // Sync final cert with VANG
          _syncCertWithVang(b64FinalCert, ca, x509Cert.getSerialNumber().toString(16));
          createState.get().setFinalCertSynced(true);

          RamenAudit.logIssuedCert(ca, x509Cert.getSubjectDN().getName(), x509Cert.getSerialNumber().toString(16), b64FinalCert);

        } else {
          b64FinalCert = requestedCaService.createCertificate(requestId, createReq.getUserDN(), createReq.getCertType(),
              createReq.getUserPKCS10(), createReq.getSubjectAltName(), caCreateReq.getGmtKeyExpiryDate(),
              finalCertExts, false);
          long elapsed2 = stopwatch.elapsed(TimeUnit.MILLISECONDS);
          
          RamenStats.log("RamenServiceImpl.createCertificate().finalCert", elapsed2-elapsed1);
          elapsed1 = elapsed2;
          x509Cert = (X509Certificate) cf.generateCertificate(
              new ByteArrayInputStream(BaseEncoding.base64().decode(b64FinalCert)));

          // Lint if linting is enabled.
          if (raInfo.getCertLinting() != LintingStatus.Disabled) {
            logger.info(CLIENT_TYPE + ": " + raInfo.getName() + " calling linting service for NoCtCert linting");
            LintingManager lintingManager = getLintingManager(raInfo);
            if (! lintingManager.lint(LintingManager.ERROR_CERTIFICATE_FAILED_LINTING, raInfo.getCertLinting(), b64FinalCert, requestId, createReq.getUserDN())) {
              throw new RaException(Status.BAD_REQUEST, LintingManager.ERROR_CERTIFICATE_FAILED_LINTING);
            }
          }

          // Sync cert with VANG
          _syncCertWithVang(b64FinalCert, ca, x509Cert.getSerialNumber().toString(16));
          createState.get().setFinalCertSynced(true);

          RamenAudit.logIssuedCert(ca, x509Cert.getSubjectDN().getName(), x509Cert.getSerialNumber().toString(16), b64FinalCert);
        }

        if (caaCheckResult != null) {
          String serialNumberHex = x509Cert.getSerialNumber().toString(16);
          caaService.saveCanIssueJsonAudit(raInfo.getName(), serialNumberHex, caaCheckResult.getJsonAudit());
          logger.info("Saved \"canIssue\" CAA audit for " + raInfo.getName() + ":" + serialNumberHex);
        }

        RACertificateResponse res = new RACertificateResponse();
        res.setRequestId(requestId);
        res.setUserCert(b64FinalCert);
        res.setSerialNumber(x509Cert.getSerialNumber().toString());
        res.setStatus(Status.OK.getStatusCode());
        SimpleDateFormat sdf = new SimpleDateFormat(RAResponse.RA_RESPONSE_DATE_FORMAT);
        String expiryString = sdf.format(x509Cert.getNotAfter());
        res.setExpiryDate(expiryString);
        
        // If the CaCreateRequest has an associated keyPair, then we generated it and must return it in a p12
        if (caCreateReq.getKeyPair() != null) {
          res.setPkcs12(buildPkcs12(caCreateReq, x509Cert, ca));
        }

        return res;
      } catch (Exception e) {
        logger.error("Unexpected error: {}", e.getMessage(), e);
        // Revoke a cert if we got far enough to have generated a cert or a precert
        CreateState cs = createState.get();
        if (cs.getB64FinalCert() != null || cs.getB64Precert() != null) {
          if (cs.getB64FinalCert() != null && cs.isFinalCertSynced()) {
            revokeB64Cert(cs.getB64FinalCert(), "Revoking b64Cert final cert");
          } else if (cs.getB64Precert() != null && cs.isPrecertSynced()) {
            revokeB64Cert(cs.getB64Precert(), "Revoking b64 precert");
          } else {
            logger.error(
                "Unable to revoke " + (cs.getB64Precert() != null && cs.getB64FinalCert() == null ? "precert" : "cert")
                    + " as it was not yet synced to VANG.");
          }
        }
        if (e instanceof RaException) {
          throw e;
        } else {
          throw new RaException(Status.INTERNAL_SERVER_ERROR, e);
        }
      }
    } finally {
      createSample.stop(createTimer);
      RamenStats.log("RamenServiceImpl.createCertificate()", stopwatch.elapsed(TimeUnit.MILLISECONDS));
      createState.remove();
    }
  }

  /*
   * TODO: Refactor this in 13.0 to a separate util method. This has grown large enough it has a decent amount of logic
   * in it and it's difficult to unit-test when it's nested as a private method within RamenServiceImpl. Too close to
   * 12.10.1 code freeze to do this refactor for 12.10.1
   *  
   * Largely lifted from com.entrust.ecs.selfservera.Pkcs12Creator.repackagePkcs12WithLinkCert, adapted for SML
   */
  private String buildPkcs12(CaCreateRequest caCreateReq, X509Certificate x509Cert, String ca)
      throws CertificateEncodingException, CertificateException, PKCSException, IOException,
      RFC2253NameParserException {

    StopWatch stopWatch = new StopWatch();
    stopWatch.start();
    CreateRequest createReq = caCreateReq.getCreateReq();
    SecureStringBuffer p12Pass = new SecureStringBuffer(createReq.getP12Password().toCharArray());
    
    // ECSPR-33065: Populate friendlyName and localKeyID
    String friendlyName = null;
    X500Principal x500Principal = x509Cert.getSubjectX500Principal();
    
    // Mobile Device (DC) uses the full subjectDN
    if (createReq.getSslCertRule().equals(SslCertType.DC.toString())) {
      friendlyName = x500Principal.toString();
      // other cert types use CN but will fallback to subjectDN if CN is null
    } else {
      RFC2253NameParser rfc2253NameParser = new RFC2253NameParser(x500Principal.getName());
      Name name = rfc2253NameParser.parse();
      String commonName = name.getRDN(ObjectID.commonName);
      if (StringUtils.isNotBlank(commonName)) {
        friendlyName = commonName;
      } else {
        friendlyName = x500Principal.toString();
      }
    }
    logger.debug("friendlyName: " + friendlyName);
    
    byte[] localKeyId = x509Cert.getSerialNumber().toByteArray();
    KeyBag keyBag = new KeyBag(caCreateReq.getKeyPair().getPrivate(), friendlyName, localKeyId);
    CertificateBag certificateBag =
        new CertificateBag(new iaik.x509.X509Certificate(x509Cert.getEncoded()), friendlyName, localKeyId);
    ArrayList<CertificateBag> certBags = new ArrayList<>();
    certBags.add(certificateBag);
    
    // lookup link cert(s)
    List<TrustChainCertificate> trustChainCertificates = new Vector<TrustChainCertificate>();
    // Chains for Mobile Device certificates contain the root

    List<TrustChainCertificate> sha2ChainCertificates = certificatePickupService.getTrustChainCertificates(ca,
        SignAlgType.SHA2.toString(), createReq.getSslCertRule().equals(SslCertType.DC.toString()));
    if (sha2ChainCertificates != null) {
      trustChainCertificates.addAll(sha2ChainCertificates);
    } else {
      logger.error("Failed to retrieve link certificate for: " + ca + ", " + SignAlgType.SHA2 + ", includeRoot: "
          + createReq.getSslCertRule().equals(SslCertType.DC.toString()));
    }

    // Add link certs to certificateBags
    if (trustChainCertificates.size() > 0) {
      for (TrustChainCertificate trustChainCertificate : trustChainCertificates) {
        iaik.x509.X509Certificate linkCert = new iaik.x509.X509Certificate(Base64
            .decodeBase64(com.entrust.eaf.utility.CertificateUtil.cleanBase64Cert(trustChainCertificate.getB64Cert())));
        CertificateBag newCaCertBag = new CertificateBag(linkCert);
        certBags.add(newCaCertBag);
      }
    }
    
    PKCS12 newPkcs12 = new PKCS12(keyBag, certBags.toArray(new CertificateBag[certBags.size()]), 2000);
    Long beforeEncrypt = stopWatch.getTime();
    newPkcs12.encrypt(p12Pass.toCharArray());
    stopWatch.stop();
    logger.debug("Encrypted P12 in " + (stopWatch.getTime() - beforeEncrypt) + " ms");
    ByteArrayOutputStream outPkcs12 = new ByteArrayOutputStream();
    try {
      newPkcs12.writeTo(outPkcs12);
    } finally {
      outPkcs12.close();
    }
    String b64Pkcs12 = new Base64(0).encodeToString(outPkcs12.toByteArray());
    
    logger.debug("Built P12 in " + stopWatch.getTime() + " ms");

    return b64Pkcs12;
  }

  public RARevokeResponse revokeCertificate() {

    Stopwatch stopwatch = Stopwatch.createStarted();
    try {
      setMdcRequestId(String.valueOf(revokeRequest.getTrackingID()));
      logger.info("Revoke request for \'{}\': {}", ca, revokeRequest);

      checkDuplicateRevokeRequest(revokeRequest.getUserDN(), revokeRequest.getCertificateSerialNumber());

      CaService requestedCaService = getRequestedCaService(ca);
      Date revocationDate = null;

      if (feederApiClient.isEnabled()) {

        RAInfo2 raInfo = certificateDataService.getRAInfoByName(requestedCaService.getCaName());

        VangRevokeRequest.Certificate revokeReqCert = new VangRevokeRequest.Certificate();

        if (requestedCaService.isAffirmTrust()) {
          VangRevokeRequest.AkiSerialNumber akiSerialNumber =
              buildAkiSerialNum(requestedCaService.getAki(), revokeRequest.getCertificateSerialNumber());
          revokeReqCert.setAki_and_serial(akiSerialNumber);
        } else {
          // If an AKI is provided, build the revocation req by AKI & SerialNum as above)
          String aki = revokeRequest.getAki();
          if (StringUtils.isNotBlank(aki)) {
            revokeReqCert
                .setAki_and_serial(buildAkiSerialNum(aki, revokeRequest.getCertificateSerialNumber()));
          } else {
            String b64CertToRevoke = certificateService2.getCertByCaSerialNumber(raInfo.getName(),
                revokeRequest.getCertificateSerialNumber());

            if (b64CertToRevoke == null) {
              String errorMsg = String.format("Unable to find certificate for ca=%s, serialNumber=%s.",
                  raInfo.getName(), revokeRequest.getCertificateSerialNumber());
              logger.error(errorMsg);
              throw new Exception(errorMsg);
            }
            revokeReqCert.setEncoding(b64CertToRevoke);
          }
        }

        // This revocation date/time is the one we send to VANG and return to the client
        // Previously, we return the SM revocation date/time to the client
        revocationDate = new Date();
        Long invalidityDate = revokeRequest.getLastGoodDate()==null ? null :
          TimeUnit.MILLISECONDS.toSeconds(revokeRequest.getLastGoodDate().getTime());

        VangRevokeRequest revokeReq = new VangRevokeRequest();
        revokeReq.setCertificate(revokeReqCert);
        revokeReq.setReason(revokeRequest.getRevocationReason());
        revokeReq.setRevokedDate(TimeUnit.MILLISECONDS.toSeconds(revocationDate.getTime()));
        revokeReq.setInvalidityDate(invalidityDate);

        _revokeCertWithVang(revokeReq);
      }

      // When a VANG revocation succeeds but the SM revocation fails at the RA, RA clients will receive a success.
      // However, we should keep track of these SM revocation errors for followup.
      // For 12.7.1, there is no easy way to send notifications from the RA. These SM revocation errors will remain in
      // the RA logs for now.
      // TODO -- For 12.8, we can leverage the error message service that was exposed for linting errors. These SM
      // revocation errors will thus show up on the ECS Dev Support mailbox.
      // Revoke cert in CA service - SM is the only CA type that actually does work
      Date serviceRevocationDate = null;
      try {
        // Revoke attempt #1
        serviceRevocationDate = requestedCaService.revokeCertificate(Integer.toString(revokeRequest.getTrackingID()),
            revokeRequest.getUserDN(), revokeRequest.getCertificateSerialNumber(), revokeRequest.getRevocationReason(),
            revokeRequest.getRevocationText(), revokeRequest.isIssueCRL(), revokeRequest.getLastGoodDate());

        logger.info("Revoked cert in {}: {}", getCaServiceDesc(requestedCaService), serviceRevocationDate);
      } catch (Exception e1) {
        logger.error("Revoke in {} failed: {}", getCaServiceDesc(requestedCaService), e1.getMessage());
        // Revoke attempt #2
        try {
          serviceRevocationDate = requestedCaService.revokeCertificate(Integer.toString(revokeRequest.getTrackingID()),
              revokeRequest.getUserDN(), revokeRequest.getCertificateSerialNumber(), revokeRequest.getRevocationReason(),
              revokeRequest.getRevocationText(), revokeRequest.isIssueCRL(), revokeRequest.getLastGoodDate());
          logger.info("Revoked cert in {}: {}", getCaServiceDesc(requestedCaService), serviceRevocationDate);
        } catch (Exception e2) {
          logger.error("Revoke in {} failed: {}", getCaServiceDesc(requestedCaService), e2.getMessage());
          // Failed revocation in CA service is not fatal if VANG is enabled, it is otherwise
          if (!feederApiClient.isEnabled()) {
            throw e2;
          }
        }
      }

      RamenAudit.logRevoked(ca, revokeRequest.getUserDN(), revokeRequest.getCertificateSerialNumber());

      RARevokeResponse res = new RARevokeResponse();
      SimpleDateFormat sdf = new SimpleDateFormat(RAResponse.RA_RESPONSE_DATE_FORMAT);

      // Return VANG revocation date to client if enabled, service revocation date otherwise
      res.setRevocationTime(sdf.format(revocationDate!=null?revocationDate:serviceRevocationDate));
      return res;
    } catch (Exception e) {
      String msg = String.format("Failed to revoke %s [%s]", revokeRequest.getUserDN(), revokeRequest.getCertificateSerialNumber());
      RaException raException = null;
      if (e instanceof RaException) {
        raException = new RaException(Status.INTERNAL_SERVER_ERROR, msg, e.getCause());
      } else if (e instanceof IllegalArgumentException) {
        raException = new RaException(Status.BAD_REQUEST, msg, e);
      } else {
        raException = new RaException(Status.INTERNAL_SERVER_ERROR, msg, e);
      }
      throw raException;
    } finally {
      RamenStats.log("RamenServiceImpl.revokeCertificate()", stopwatch.elapsed(TimeUnit.MILLISECONDS));
    }
  }

  protected AkiSerialNumber buildAkiSerialNum(String aki, String serialNum) {
    String serialNumberHex = new BigInteger(serialNum).toString(16);
    VangRevokeRequest.AkiSerialNumber akiSerialNumber = new VangRevokeRequest.AkiSerialNumber();
    akiSerialNumber.setAki(aki);
    akiSerialNumber.setSerial(serialNumberHex);
    return akiSerialNumber;
  }

  private LintingManager getLintingManager(RAInfo2 raInfo) {
    ZLinterConfig zlConfig = new ZLinterConfig();
    zlConfig.setUrl(lintingServiceUrl);
    zlConfig.setConnectionManagerTimeout(lintingServiceConnectionManagerTimeout);
    zlConfig.setConnectionTimeout(lintingServiceConnectionTimeout);
    zlConfig.setSocketTimeout(lintingServiceSocketTimeout);
    LintingManager lintingManager = new LintingManager(zlConfig, CLIENT_TYPE, raInfo.getName(), raInfo.getIssuerCn(), cmsService);
    return lintingManager;
  }
  private Object setObjectFromProperties(String className, String baseKey) throws Exception {

    Object object = Class.forName(className).getDeclaredConstructor().newInstance();
    Map<String, String> propertyMap = BeanUtils.describe(object);
    Set<String> fieldNames = propertyMap.keySet();

    Object instance = Class.forName(className).getDeclaredConstructor().newInstance();
    for (String fieldName : fieldNames) {
      String fieldKey = baseKey + "." + fieldName;
      String fieldValue = env.getProperty(fieldKey);
      if (fieldValue!=null) {
        BeanUtils.setProperty(instance, fieldName, fieldValue);
      }
    }
    return instance;
  }

  private void setMdcRequestId(String requestId) {
    // The format here must match the getMdcRequestId() method
    MDC.put(MDC_REQUEST_ID, String.format("[%s] ", requestId));
  }

  public static String getMdcRequestId() {
    String formattedRequestId = MDC.get(MDC_REQUEST_ID);
    return formattedRequestId == null ? null : formattedRequestId.substring(1, formattedRequestId.length() - 2);
  }

  private void initCaServices() throws Exception {
    for (CaServiceConfig caServiceConfig : caServicesConfig.getCaServiceConfigs()) {
      String caServiceName = caServiceConfig.getName();
      String serviceKey = CaServicesConfig.propertyKey + "." + caServiceName;
      String baseKey = null;
      switch (caServiceConfig.getType()) {
        case CAGW:
          baseKey = serviceKey + "." + CagwConfig.CONFIG_KEY;
          CagwConfig cagwConfig = (CagwConfig) setObjectFromProperties(CagwConfig.class.getName(), baseKey);
          logger.info("Instantiating CA service [{}]: {}", caServiceConfig, cagwConfig );
          CagwCaService cagwCaService = new CagwCaService(cagwConfig);
          caServices.put(caServiceConfig.getName(), cagwCaService);
          logger.info("Instantiated CA service [{}].", caServiceName);
          break;
        case CA_POOL:
          CaPoolConfig caPoolConfig = new CaPoolConfig();
          caPoolConfig.setCaName(env.getProperty(serviceKey + "." + CaPoolConfig.CA_NAME_KEY));
          String casProperty = env.getProperty(serviceKey + "." + CaPoolConfig.CONFIG_KEY + "." + CaPoolConfig.CAS_KEY);
          String[] caNames = casProperty.split(",");
          for (String caName : caNames) {
            baseKey = serviceKey + "." + CaPoolConfig.CONFIG_KEY + "." + caName;
            ToolkitCaConfig caConfig = (ToolkitCaConfig) setObjectFromProperties(ToolkitCaConfig.class.getName(), baseKey);
            caPoolConfig.getCaConfigs().add(caConfig);
          }
          logger.info("Instantiating CA service [{}]: {}", caServiceConfig, caPoolConfig);
          ToolkitCaPoolService toolkitCaPoolService = new ToolkitCaPoolService(caPoolConfig);
          caServices.put(caServiceConfig.getName(), toolkitCaPoolService);
          logger.info("Instantiated CA service [{}].", caServiceName);
          break;
        case CA:
          baseKey = serviceKey;
          ToolkitCaConfig caConfig = (ToolkitCaConfig) setObjectFromProperties(ToolkitCaConfig.class.getName(), baseKey);
          logger.info("Instantiating CA service [{}]: {}", caServiceConfig, caConfig);
          ToolkitCaService toolkitCaService = new ToolkitCaService(caConfig);
          caServices.put(caServiceConfig.getName(), toolkitCaService);
          logger.info("Instantiated CA service [{}].", caServiceName);
          break;
        case SML:
          baseKey = serviceKey + "." + SmlConfig.CONFIG_KEY;
          SmlConfig smlConfig = (SmlConfig) setObjectFromProperties(SmlConfig.class.getName(), baseKey);
          logger.info("Instantiating CA service [{}]: {}", caServiceConfig, smlConfig);

          SmlCaService smlCaService = new SmlCaService(smlConfig, env);
          smlCaService.setCertificateDataService2(certificateDataService2);
          smlCaService.setCertificateService2(certificateService2);
          caServices.put(caServiceConfig.getName(), smlCaService);
          logger.info("Instantiated CA service [{}].", caServiceName);
          break;
        default:
          throw new ServletException("Unsupported CA type: " + caServiceConfig.getType()) ;
      }
    }
  }

  private SslCertType sslCertTypeFromRule(String requestId, String sslCertRule) throws RaException {

    try {
      SslCertType sslCertType = SslCertType.valueOf(sslCertRule);
      if (sslCertType == SslCertType.NoValidation) {
        throw new IllegalArgumentException();
      }
      return sslCertType;
    } catch (IllegalArgumentException e) {
      throw new RaException(requestId, Status.BAD_REQUEST,
          "\'" + sslCertRule + "\' is not a supported Entrust SSL certificate type");
    }
  }

  private void validateCreateReq(String requestId, String orderId, String accountNumber, int domainId,
      String trackingId, String csr, String subjectDN, String sanString, SslCertType sslCertType, Date expiryDate, boolean isAffirmTrust)
      throws RaException {
    // Ensure that the request is valid
    Timer.Sample validationSample = Timer.start(Metrics.globalRegistry);
    try {
      if (!skipValidation) {
        // Check that there are no leading or trailing spaces
        // JastkUtil.checkNameForLeadingTrailingSpace(name);
        
        // ECSPR-32041 Ensure expiryDate is not NULL
        if (expiryDate == null) {
          throw new RaException(requestId, Status.BAD_REQUEST, "expiryDate cannot be null");
        }

        if (!isAffirmTrust) {
          logger.info("Calling CertificateDataService.validate2WithCsrChecks()...");
          ValidationResult validationResult;
          try {
            validationResult = certificateDataService.validate2WithRaCsrChecks(csr, orderId, trackingId,
                accountNumber, sslCertType, null /* resellerNumParam */, null/* clientDn */,
                subjectDN, sanString, expiryDate, domainId);
          } finally {

          }
          if (validationResult.isMessagesPresent()) {
            msgArrayToRaException("", validationResult.getValidationMessages(), requestId);
          }
        } else {
          Boolean isOnHardStopList = customerService.isOnAffirmTrustHardStopList(subjectDN, sanString);
          if(isOnHardStopList) {
            // ***** This error pattern is expected by the AffirmTrust client application. *****
            // ***** DO NOT change error string
            String message = "ERROR: Forbidden string appeared in dn or sans input.";
            throw new RaException(requestId, Status.BAD_REQUEST, message);
          }
        }
      } else {
        logger.warn("***** DEV TEST ONLY -- SKIPPING ORDER/ACCOUNT VALIDATION *****");
      }
    } catch (Exception e) {
      logger.info("Unexpected validation exception: " + e.getMessage(), e);
      String message = "The certificate request and associated information is not valid. " + e.getMessage();

      // Print stack trace for NullPointerException
      if (e instanceof NullPointerException) {
        message = NullPointerException.class.getSimpleName() + ": " + ExceptionUtils.getStackTrace(e);
      }
      throw new RaException(requestId, Status.BAD_REQUEST, message);
    } finally {
      validationSample.stop(validateTimer);
    }
  }

  private void msgArrayToRaException(String messageHeader, Message[] validationMessages, String requestId)
      throws RaException {
    String errorMessage =
        messageHeader + Arrays.stream(validationMessages).map(o -> o.getMessageKey()).collect(Collectors.joining(" "));
    throw new RaException(requestId, Status.BAD_REQUEST, errorMessage);
  }

  private void checkDuplicateCreateRequest(CreateRequest caCreateReq) throws RaException {
    try {
      
      // Check for duplicate requests based on the hex string representation of the hashCode of the request 
      String hashString = caCreateReq.toHashString();
      if( createReqCacheLock.tryLock(CACHE_LOCK_ACQUIRE_TIMEOUT, TimeUnit.SECONDS) ) {
        Boolean cached = createReqCache.getIfPresent(hashString);
        logger.info( createReqCache.size() + ", " + createReqCache.stats() );

        // Check for duplicate request
        // We check whether the key is in the cache - the cached value is not used
        if( cached != null ) {
          String errorMsg = String.format("Duplicate create request detected for userDN: %s, request hash: %s",
              caCreateReq.getUserDN(), hashString);
          logger.info(errorMsg);
          throw new RaException(Status.BAD_REQUEST, errorMsg);
        } else {
          createReqCache.put(hashString, true);
        }
      }
    } catch (InterruptedException e) {
        String errorMsg = "Unable to acquire lock on createReqCache";
        logger.info(errorMsg);
        throw new RaException(Status.INTERNAL_SERVER_ERROR, errorMsg);
    }finally{
      //release lock
      createReqCacheLock.unlock();
    }
  }

  private void checkDuplicateRevokeRequest(String inUserDN, String serialNumber) throws RaException {
    try {
      if( revokeReqCacheLock.tryLock(CACHE_LOCK_ACQUIRE_TIMEOUT, TimeUnit.SECONDS) ) {
        String cachedSerialNumber = revokeReqCache.getIfPresent(inUserDN);
        logger.info( revokeReqCache.size() + ", " + revokeReqCache.stats() );

        if(cachedSerialNumber!=null && cachedSerialNumber.equalsIgnoreCase(serialNumber) ) {
          String errorMsg = "Duplicate revoke request detected for userDN: " + inUserDN + ", serialNumber: " + serialNumber;
          logger.info(errorMsg);
          throw new RaException(Status.BAD_REQUEST, errorMsg);
        } else {
          revokeReqCache.put(inUserDN, serialNumber);
        }
      }
    } catch (InterruptedException e) {
      String errorMsg = "Unable to acquire lock on revokeReqCache";
      logger.info(errorMsg);
      throw new RaException(Status.INTERNAL_SERVER_ERROR, errorMsg);
    }finally{
      //release lock
      revokeReqCacheLock.unlock();
    }
  }

  private void addCustomExtensions(X509Extensions extensions, X509Extensions customExtensions)
      throws X509ExtensionException {
    List<V3Extension> customExtensionsList = EnumerationUtils.toList(customExtensions.listExtensions());
    for (V3Extension customExtension : customExtensionsList) {
      extensions.addExtension(customExtension);
    }
  }

  public RAResponse syncCertWithVang(VangSyncRequest syncRequest) {

    String b64FinalCert = CertificateUtil.cleanBase64Cert(syncRequest.getCert());
    String caName = syncRequest.getCaName();
    String serialNumberHex = syncRequest.getSerialNumberHex();

    String requestId = null;
    if (syncRequest.getRequestId() == null) {
      requestId = Integer.toString(Math.abs(syncRequest.hashCode()), 16).toUpperCase();
    } else {
      requestId = syncRequest.getRequestId();
    }
    setMdcRequestId(requestId);
    RAResponse vangResponse = new RAResponse(requestId, null, null);
    try {
      _syncCertWithVang(b64FinalCert, caName, serialNumberHex);
      vangResponse.setUserMessage("Success");
    } catch (RaException e) {
      logger.error("Error trying to sync cert from ca {} with serialNumber {} with vang", caName, serialNumberHex);
      vangResponse.setStatus(e.getStatus().getStatusCode());
      vangResponse.setUserMessage(e.getMessage());
      vangResponse.setStatus(e.getStatus().getStatusCode());
    }
    return vangResponse;
  }

  private void _syncCertWithVang(String b64Cert, String caName, String serialNumberHex) throws RaException {

    Timer.Sample syncSample = Timer.start(Metrics.globalRegistry);

    try {
      // Sync cert into VANG Feeder API
      if (feederApiClient != null && feederApiClient.isEnabled()) {
        try {
          FeederResponse res = feederApiClient.sync(b64Cert);

          if(res.getStatus()==Status.OK.getStatusCode()) {
            logger.info("Synced cert with VANG");
          } else {
            String errorMsg = String.format("[%d] %s", res.getStatus(), res.getVangResponse().getError().trim());
            throw new RaException(Status.fromStatusCode(res.getStatus()), errorMsg);
          }
        } catch (Exception e) {

          // If we are ignoring VANG errors, log error and continue.
          // If not, throw exception for it to be handled normally.

          // Default to a 500 INTERNAL_SERVER_ERROR exception
          Status status = Status.INTERNAL_SERVER_ERROR;
          if ( e instanceof RaException) {
            status = ((RaException) e).getStatus();
          }

          String errorMsg = "VANG error" + (ignoreVangErrors ? " [ignoring]" : "") + ": Sync failed: " + e.getMessage();
          try {
            OrphanedCert orphanedCert =
                new OrphanedCert(caName, serialNumberHex, false, b64Cert, OrphanedCertEvent.VANG, !ignoreVangErrors, false, false);
            certificateService2.newOrphanedCert(orphanedCert);
          } catch (EcsFault ef) {
            // This is a "secondary" exception that we just want to log.
            // The main condition that we need to process is the caught exception
            logger.error(errorMsg, ef);
          }
          if (ignoreVangErrors) {
            logger.error(errorMsg, e);
          } else {
            logger.error(errorMsg);
            throw new RaException(status, errorMsg, e);
          }
        }
      }
    } finally {
      syncSample.stop(syncTimer);
    }
  }

  public RARevokeResponse revokeCertWithVang(VangRevokeRequest revokeReq) {
    RARevokeResponse vangResponse = new RARevokeResponse();

    setMdcRequestId(revokeReq.getRequestId());
    vangResponse.setRequestId(revokeReq.getRequestId());
    long revokedDate = -1;
    try {
      if (revokeReq.getRevokedDate() == null) {
        revokedDate = new Date().getTime();
      } else {
        revokedDate = revokeReq.getRevokedDate();
      }

      // Convert times to seconds
      revokeReq.setRevokedDate(TimeUnit.MILLISECONDS.toSeconds(revokedDate));
      revokeReq.setInvalidityDate(revokeReq.getInvalidityDate() == null ? null
              : TimeUnit.MILLISECONDS.toSeconds(revokeReq.getInvalidityDate()));
      _revokeCertWithVang(revokeReq);

      vangResponse.setUserMessage("Success");
      SimpleDateFormat sdf = new SimpleDateFormat(RAResponse.RA_RESPONSE_DATE_FORMAT);

      // Return VANG revocation date to client if enabled, service revocation date otherwise
      vangResponse.setRevocationTime(sdf.format(new Date(revokedDate)));
    } catch (RaException e) {
      logger.error("Error trying to revoke cert with vang");
      vangResponse.setStatus(e.getStatus().getStatusCode());
      vangResponse.setUserMessage(e.getMessage());
      vangResponse.setStatus(e.getStatus().getStatusCode());
    }
    return vangResponse;
  }

  private void _revokeCertWithVang(com.entrust.ecs.vang.VangRevokeRequest revokeReq)
      throws RaException {

    if (feederApiClient != null && feederApiClient.isEnabled()) {

      logger.info("Revoke cert with VANG: {}", revokeReq);

      try {
        FeederResponse res = feederApiClient.revoke(revokeReq);

        if(res.getStatus()==Status.OK.getStatusCode()) {
          logger.info("Revoked cert with VANG");
        } else {
          String errorMsg = String.format("[%d] %s.%n%s", res.getStatus(), res.getVangResponse().getError().trim(),
              new Gson().toJson(revokeReq));
          throw new RaException(Status.fromStatusCode(res.getStatus()), errorMsg);
        }
      } catch (Exception e) {
        // If we are ignoring VANG errors, log error and continue.
        // If not, throw exception for it to be handled normally.

        // Default to a 500 INTERNAL_SERVER_ERROR exception
        Status status = Status.INTERNAL_SERVER_ERROR;
        if ( e instanceof RaException) {
          status = ((RaException) e).getStatus();
        }

        String errorMsg = "VANG error" + (ignoreVangErrors ? " [ignoring]" : "") + ": Revocation failed: " + e.getMessage();
        if (ignoreVangErrors) {
          logger.error(errorMsg, e);
        } else {
          logger.error(errorMsg);
          throw new RaException(status, errorMsg, e);
        }
      }
    }
  }
  
  private void revokeB64Cert(String b64Cert, String msg) {
    logger.error(msg);
    VangRevokeRequest.Certificate revokeReqCert = new VangRevokeRequest.Certificate();
    revokeReqCert.setEncoding(b64Cert);
    VangRevokeRequest revokeReq = new VangRevokeRequest();
    revokeReq.setCertificate(revokeReqCert);
    revokeReq.setReason(EcsCRLReason.superseded.getReason());
    revokeReq.setRevokedDate(TimeUnit.MILLISECONDS.toSeconds(new Date().getTime()));
    try {
    _revokeCertWithVang(revokeReq);
    } catch (RaException e) {
      logger.error("Error revoking cert or precert", e);
    }
    return;
  }

  private CaaCheckResult processCaaCheckResult(RAInfo2 raInfo, String san, String trackingId, String accountNumber, String orderId)
      throws RaException {

    Timer.Sample caaSample = Timer.start(Metrics.globalRegistry);
    try {
      // CAA checks
      boolean checkCAA = raInfo.isCheckCAA();
      boolean enforceCAA = raInfo.isEnforceCAA();
      logger.info("CAA check is " + (checkCAA ? "enabled" : "disabled") + " for this RA(" + raInfo.getName() + ")");

      CaaCheckResult caaCheckResult = null;
      if (checkCAA) {
        caaCheckResult = checkCAA(san, raInfo.getCaaIssuers(), accountNumber, enforceCAA);
        if (caaCheckResult!=null && caaCheckResult.getErrorMessage() != null) {
          Integer auditTrackingId = null;
          Integer auditOrderId = null;

          try {
            auditTrackingId = Integer.valueOf(trackingId);
          } catch (RuntimeException re) {
            // Continue with null
          }

          try {
            auditOrderId = Integer.valueOf(orderId);
          } catch (RuntimeException re) {
            // Continue with null
          }

          caaService.saveRejectedJsonAudit(raInfo.getName(), auditTrackingId, accountNumber, auditOrderId, caaCheckResult.getJsonAudit());
          logger.info(String.format("Saved rejected CAA audit for: %s, %d, %s, %d",
              raInfo.getName(), auditTrackingId, accountNumber, auditOrderId));

          if(enforceCAA) {
            throw new RaException(Status.BAD_REQUEST, caaCheckResult.getErrorMessage());
          } else {
            logger.info("Failed CAA check is not enforced for this RA(" + raInfo.getName() + ")");
          }
        }
      }
      return caaCheckResult;
    } finally {
      caaSample.stop(caaTimer);
    }
  }

  private CaaCheckResult checkCAA(String subAltName, String allowedIssuers, String accountNumber, boolean sendRejectNotification) {

    logger.info(String.format("checkCAA(subAltName=%s, allowedIssuers=%s)", subAltName, allowedIssuers));

    String commaSeparatedDomains = SanString.toSeparatedValues(subAltName, SanStringType.dNSName, ",", true);

    if (commaSeparatedDomains.isEmpty()) {
      // Nothing to check -- e.g. SAN with a single IP specified as a dNSName type
      logger.info("No domains for CAA checks.");
      return null;

    } else {

      logger.info("Do CAA checks for: " + commaSeparatedDomains);
      String jsonAudit = caaService.getJsonAuditsForAllowedIssuers2(commaSeparatedDomains, allowedIssuers, false, accountNumber, sendRejectNotification);
      CaaCheckResult caaCheckResult = new CaaCheckResult(jsonAudit, null);

      Type listType = new TypeToken<ArrayList<Rfc6844Result>>(){}.getType();
      Gson gson = GsonUtil.getGsonBuilder(false).setDateFormat(Rfc6844Result.SOAP_DATE_FORMAT).create();
      List<Rfc6844Result> rfc6844Results = gson.fromJson(jsonAudit, listType);

      List<String> caaBlockedDomainsAndStatus = new ArrayList<>();

      for (Rfc6844Result rfc6844Result : rfc6844Results) {
        if (!rfc6844Result.isCanIssue()) {
          caaBlockedDomainsAndStatus.add(rfc6844Result.getDomain() + " [" + rfc6844Result.getStatus() + "]");
        }
      }

      if (caaBlockedDomainsAndStatus.size() != 0) {

        StringBuilder errMsg = new StringBuilder("CA authorization check failed for the following domains: ");
        String details = ArrayUtils.toString(caaBlockedDomainsAndStatus);
        // Strip leading and trailing square bracket set
        if(details.startsWith("[") && details.endsWith("]")) {
          details = details.substring(1, details.length()-1);
        }
        errMsg.append(details);
        caaCheckResult.setErrorMessage(errMsg.toString());
      }
      return caaCheckResult;
    }
  }

  public void setCreateReqCache(Cache<String, Boolean> createReqCache) {
    this.createReqCache = createReqCache;
  }

  public void setRevokeReqCache(Cache<String, String> revokeReqCache) {
    this.revokeReqCache = revokeReqCache;
  }

  public FeederApiClient getFeederApiClient() {
    return feederApiClient;
  }

  public void setFeederApiClient(FeederApiClient feederApiClient) {
    this.feederApiClient = feederApiClient;
  }
  
  public static void setCreateState(ThreadLocal<CreateState> createState) {
    RamenServiceImpl.createState = createState;
  }
  
  public static ThreadLocal<CreateState> getCreateState() {
    return RamenServiceImpl.createState;
  }

  private void runProcessors(CreateReqModifier ...createReqModifiers ) throws RaException {
    for(CreateReqModifier modifier : createReqModifiers) {
      modifier.process();
    }
  }

  private String getCaServiceDesc(CaService caService) {
    return String.format("%s[%s] CA service", caService.getCaName(), caService.getType());
  }

  private CaService getRequestedCaService(String ca) throws RaException {
    CaService requestedCaService = Strings.isNullOrEmpty(ca) ? defaultCaService : caServices.get(ca);
    if (requestedCaService==null) {
      throw new RaException(Status.NOT_FOUND, String.format("\'%s\' is not a configured CA.", ca));
    }
    return requestedCaService;
  }

  private Timer createReqTimer(String tagValue) {
    return Timer.builder("ramen.create.request.duration").tag("phase", tagValue).register(Metrics.globalRegistry);
  }

}
