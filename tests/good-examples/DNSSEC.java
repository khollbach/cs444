/*      */ package org.xbill.DNS;
/*      */ 
/*      */ import java.io.IOException;
/*      */ import java.math.BigInteger;
/*      */ import java.security.GeneralSecurityException;
/*      */ import java.security.KeyFactory;
/*      */ import java.security.MessageDigest;
/*      */ import java.security.NoSuchAlgorithmException;
/*      */ import java.security.PrivateKey;
/*      */ import java.security.PublicKey;
/*      */ import java.security.Signature;
/*      */ import java.security.interfaces.DSAPrivateKey;
/*      */ import java.security.interfaces.DSAPublicKey;
/*      */ import java.security.interfaces.ECPrivateKey;
/*      */ import java.security.interfaces.ECPublicKey;
/*      */ import java.security.interfaces.RSAPrivateKey;
/*      */ import java.security.interfaces.RSAPublicKey;
/*      */ import java.security.spec.DSAPublicKeySpec;
/*      */ import java.security.spec.ECPoint;
/*      */ import java.security.spec.ECPublicKeySpec;
/*      */ import java.security.spec.RSAPublicKeySpec;
/*      */ import java.util.Arrays;
/*      */ import java.util.Date;
/*      */ import java.util.Iterator;
/*      */ import org.xbill.DNS.DNSSEC.DNSSECException;
/*      */ import org.xbill.DNS.DNSSEC.ECKeyInfo;
/*      */ import org.xbill.DNS.DNSSEC.IncompatibleKeyException;
/*      */ import org.xbill.DNS.DNSSEC.KeyMismatchException;
/*      */ import org.xbill.DNS.DNSSEC.MalformedKeyException;
/*      */ import org.xbill.DNS.DNSSEC.NoSignatureException;
/*      */ import org.xbill.DNS.DNSSEC.SignatureExpiredException;
/*      */ import org.xbill.DNS.DNSSEC.SignatureNotYetValidException;
/*      */ import org.xbill.DNS.DNSSEC.SignatureVerificationException;
/*      */ import org.xbill.DNS.DNSSEC.UnsupportedAlgorithmException;
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ public class DNSSEC {
/*      */    private static final ECKeyInfo ECDSA_P256 = new ECKeyInfo(32, "FFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFF", "FFFFFFFF00000001000000000000000000000000FFFFFFFFFFFFFFFFFFFFFFFC", "5AC635D8AA3A93E7B3EBBD55769886BC651D06B0CC53B0F63BCE3C3E27D2604B", "6B17D1F2E12C4247F8BCE6E563A440F277037D812DEB33A0F4A13945D898C296", "4FE342E2FE1A7F9B8EE7EB4A7C0F9E162BCE33576B315ECECBB6406837BF51F5", "FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551");
/*      */    private static final ECKeyInfo ECDSA_P384 = new ECKeyInfo(48, "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFFFF0000000000000000FFFFFFFF", "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFFFF0000000000000000FFFFFFFC", "B3312FA7E23EE7E4988E056BE3F82D19181D9C6EFE8141120314088F5013875AC656398D8A2ED19D2A85C8EDD3EC2AEF", "AA87CA22BE8B05378EB1C71EF320AD746E1D3B628BA79B9859F741E082542A385502F25DBF55296C3A545E3872760AB7", "3617DE4A96262C6F5D9E98BF9292DC29F8F41DBD289A147CE9DA3113B5F0B8C00A60B1CE1D7E819D7A431D7C90EA0E5F", "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFC7634D81F4372DDF581A0DB248B0A77AECEC196ACCC52973");
/*      */    private static final int ASN1_SEQ = 48;
/*      */    private static final int ASN1_INT = 2;
/*      */    private static final int DSA_LEN = 20;
/*      */ 
/*      */    private static void digestSIG(DNSOutput out, SIGBase sig) {
/*  114 */       out.writeU16(sig.getTypeCovered());
/*  115 */       out.writeU8(sig.getAlgorithm());
/*  116 */       out.writeU8(sig.getLabels());
/*  117 */       out.writeU32(sig.getOrigTTL());
/*  118 */       out.writeU32(sig.getExpire().getTime() / 1000L);
/*  119 */       out.writeU32(sig.getTimeSigned().getTime() / 1000L);
/*  120 */       out.writeU16(sig.getFootprint());
/*  121 */       sig.getSigner().toWireCanonical(out);
/*  122 */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    public static byte[] digestRRset(RRSIGRecord rrsig, RRset rrset) {
/*  134 */       DNSOutput out = new DNSOutput();
/*  135 */       digestSIG(out, rrsig);
/*      */ 
/*  137 */       int size = rrset.size();
/*  138 */       Record[] records = new Record[size];
/*      */ 
/*  140 */       Iterator it = rrset.rrs();
/*  141 */       Name name = rrset.getName();
/*  142 */       Name wild = null;
/*  143 */       int sigLabels = rrsig.getLabels() + 1;
/*  144 */       if (name.labels() > sigLabels) {
/*  145 */          wild = name.wild(name.labels() - sigLabels);      }
/*  146 */       while(it.hasNext()) {
/*  147 */          --size;         records[size] = (Record)it.next();      }
/*  148 */       Arrays.sort(records);
/*      */ 
/*  150 */       DNSOutput header = new DNSOutput();
/*  151 */       if (wild != null) {
/*  152 */          wild.toWireCanonical(header);
/*      */       } else {
/*  154 */          name.toWireCanonical(header);      }
/*  155 */       header.writeU16(rrset.getType());
/*  156 */       header.writeU16(rrset.getDClass());
/*  157 */       header.writeU32(rrsig.getOrigTTL());
/*  158 */       for(int i = 0; i < records.length; ++i) {
/*  159 */          out.writeByteArray(header.toByteArray());
/*  160 */          int lengthPosition = out.current();
/*  161 */          out.writeU16(0);
/*  162 */          out.writeByteArray(records[i].rdataToWireCanonical());
/*  163 */          int rrlength = out.current() - lengthPosition - 2;
/*  164 */          out.save();
/*  165 */          out.jump(lengthPosition);
/*  166 */          out.writeU16(rrlength);
/*  167 */          out.restore();
/*      */       }
/*  169 */       return out.toByteArray();
/*      */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    public static byte[] digestMessage(SIGRecord sig, Message msg, byte[] previous) {
/*  183 */       DNSOutput out = new DNSOutput();
/*  184 */       digestSIG(out, sig);
/*      */ 
/*  186 */       if (previous != null) {
/*  187 */          out.writeByteArray(previous);
/*      */       }
/*  189 */       msg.toWire(out);
/*  190 */       return out.toByteArray();
/*      */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    private static int BigIntegerLength(BigInteger i) {
/*  328 */       return (i.bitLength() + 7) / 8;
/*      */    }
/*      */ 
/*      */ 
/*      */    private static BigInteger readBigInteger(DNSInput in, int len) throws IOException {
/*  333 */       byte[] b = in.readByteArray(len);
/*  334 */       return new BigInteger(1, b);
/*      */    }
/*      */ 
/*      */ 
/*      */    private static BigInteger readBigInteger(DNSInput in) {
/*  339 */       byte[] b = in.readByteArray();
/*  340 */       return new BigInteger(1, b);
/*      */    }
/*      */ 
/*      */ 
/*      */    private static void writeBigInteger(DNSOutput out, BigInteger val) {
/*  345 */       byte[] b = val.toByteArray();
/*  346 */       if (b[0] == 0) {
/*  347 */          out.writeByteArray(b, 1, b.length - 1);
/*      */       } else {
/*  349 */          out.writeByteArray(b);      }
/*  350 */    }
/*      */ 
/*      */ 
/*      */    private static PublicKey toRSAPublicKey(KEYBase r) throws IOException, GeneralSecurityException {
/*  354 */       DNSInput in = new DNSInput(r.getKey());
/*  355 */       int exponentLength = in.readU8();
/*  356 */       if (exponentLength == 0) {
/*  357 */          exponentLength = in.readU16();      }
/*  358 */       BigInteger exponent = readBigInteger(in, exponentLength);
/*  359 */       BigInteger modulus = readBigInteger(in);
/*      */ 
/*  361 */       KeyFactory factory = KeyFactory.getInstance("RSA");
/*  362 */       return factory.generatePublic(new RSAPublicKeySpec(modulus, exponent));
/*      */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    private static PublicKey toDSAPublicKey(KEYBase r) throws IOException, GeneralSecurityException, MalformedKeyException {
/*  369 */       DNSInput in = new DNSInput(r.getKey());
/*      */ 
/*  371 */       int t = in.readU8();
/*  372 */       if (t > 8) {
/*  373 */          throw new MalformedKeyException(r);
/*      */       } else {
/*  375 */          BigInteger q = readBigInteger(in, 20);
/*  376 */          BigInteger p = readBigInteger(in, 64 + t * 8);
/*  377 */          BigInteger g = readBigInteger(in, 64 + t * 8);
/*  378 */          BigInteger y = readBigInteger(in, 64 + t * 8);
/*      */ 
/*  380 */          KeyFactory factory = KeyFactory.getInstance("DSA");
/*  381 */          return factory.generatePublic(new DSAPublicKeySpec(y, p, q, g));
/*      */       }
/*      */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    private static PublicKey toECDSAPublicKey(KEYBase r, ECKeyInfo keyinfo) throws IOException, GeneralSecurityException, MalformedKeyException {
/*  427 */       DNSInput in = new DNSInput(r.getKey());
/*      */ 
/*      */ 
/*  430 */       BigInteger x = readBigInteger(in, keyinfo.length);
/*  431 */       BigInteger y = readBigInteger(in, keyinfo.length);
/*  432 */       ECPoint q = new ECPoint(x, y);
/*      */ 
/*  434 */       KeyFactory factory = KeyFactory.getInstance("EC");
/*  435 */       return factory.generatePublic(new ECPublicKeySpec(q, keyinfo.spec));
/*      */    }
/*      */ 
/*      */ 
/*      */ 
/*      */    static PublicKey toPublicKey(KEYBase r) throws DNSSECException {
/*  441 */       int alg = r.getAlgorithm();
/*      */       try {
/*  443 */          switch(alg) {
/*      */          case 1:
/*      */          case 5:
/*      */          case 7:
/*      */          case 8:
/*      */          case 10:
/*  449 */             return toRSAPublicKey(r);
/*      */          case 3:
/*      */          case 6:         case 2:
/*  452 */             return toDSAPublicKey(r);
/*      */          case 4:         case 13:         case 9:
/*  454 */             return toECDSAPublicKey(r, ECDSA_P256);
/*      */          case 11:         case 14:         case 12:
/*  456 */             return toECDSAPublicKey(r, ECDSA_P384);
/*      */          default:
/*  458 */             throw new UnsupportedAlgorithmException(alg);
/*      */ 
/*      */          }
/*  461 */       } catch (IOException var3) {
/*  462 */          throw new MalformedKeyException(r);
/*      */ 
/*  464 */       } catch (GeneralSecurityException var4) {
/*  465 */          throw new DNSSECException(var4.toString());
/*      */       }
/*      */    }
/*      */ 
/*      */ 
/*      */    private static byte[] fromRSAPublicKey(RSAPublicKey key) {
/*  471 */       DNSOutput out = new DNSOutput();
/*  472 */       BigInteger exponent = key.getPublicExponent();
/*  473 */       BigInteger modulus = key.getModulus();
/*  474 */       int exponentLength = BigIntegerLength(exponent);
/*      */ 
/*  476 */       if (exponentLength < 256) {
/*  477 */          out.writeU8(exponentLength);
/*      */       } else {
/*  479 */          out.writeU8(0);
/*  480 */          out.writeU16(exponentLength);
/*      */       }
/*  482 */       writeBigInteger(out, exponent);
/*  483 */       writeBigInteger(out, modulus);
/*      */ 
/*  485 */       return out.toByteArray();
/*      */    }
/*      */ 
/*      */ 
/*      */    private static byte[] fromDSAPublicKey(DSAPublicKey key) {
/*  490 */       DNSOutput out = new DNSOutput();
/*  491 */       BigInteger q = key.getParams().getQ();
/*  492 */       BigInteger p = key.getParams().getP();
/*  493 */       BigInteger g = key.getParams().getG();
/*  494 */       BigInteger y = key.getY();
/*  495 */       int t = (p.toByteArray().length - 64) / 8;
/*      */ 
/*  497 */       out.writeU8(t);
/*  498 */       writeBigInteger(out, q);
/*  499 */       writeBigInteger(out, p);
/*  500 */       writeBigInteger(out, g);
/*  501 */       writeBigInteger(out, y);
/*      */ 
/*  503 */       return out.toByteArray();
/*      */    }
/*      */ 
/*      */ 
/*      */    private static byte[] fromECDSAPublicKey(ECPublicKey key) {
/*  508 */       DNSOutput out = new DNSOutput();
/*      */ 
/*  510 */       BigInteger x = key.getW().getAffineX();
/*  511 */       BigInteger y = key.getW().getAffineY();
/*      */ 
/*  513 */       writeBigInteger(out, x);
/*  514 */       writeBigInteger(out, y);
/*      */ 
/*  516 */       return out.toByteArray();
/*      */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    static byte[] fromPublicKey(PublicKey key, int alg) throws DNSSECException {
/*  524 */       switch(alg) {
/*      */       case 1:
/*      */       case 5:
/*      */       case 7:
/*      */       case 8:
/*      */       case 10:
/*  530 */          if (!(key instanceof RSAPublicKey)) {
/*  531 */             throw new IncompatibleKeyException();         }
/*  532 */          return fromRSAPublicKey((RSAPublicKey)key);
/*      */       case 3:
/*      */       case 6:
/*  535 */          if (!(key instanceof DSAPublicKey)) {
/*  536 */             throw new IncompatibleKeyException();         }
/*  537 */          return fromDSAPublicKey((DSAPublicKey)key);
/*      */       case 2:
/*      */       case 4:      case 13:      case 14:      case 9:
/*  540 */          if (!(key instanceof ECPublicKey)) {      case 11:
/*  541 */             throw new IncompatibleKeyException();         } else {      case 12:
/*  542 */             return fromECDSAPublicKey((ECPublicKey)key);
/*      */       default:
/*  544 */          throw new UnsupportedAlgorithmException(alg);
/*      */          }
/*      */       }
/*      */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    public static String algString(int alg) throws UnsupportedAlgorithmException {
/*  555 */       switch(alg) {
/*      */       case 1:
/*  557 */          return "MD5withRSA";
/*      */       case 3:
/*      */       case 6:
/*  560 */          return "SHA1withDSA";
/*      */       case 5:
/*      */       case 7:
/*  563 */          return "SHA1withRSA";
/*      */       case 8:
/*  565 */          return "SHA256withRSA";
/*      */       case 10:      case 2:
/*  567 */          return "SHA512withRSA";
/*      */       case 4:      case 13:      case 9:
/*  569 */          return "SHA256withECDSA";
/*      */       case 11:      case 14:      case 12:
/*  571 */          return "SHA384withECDSA";
/*      */       default:
/*  573 */          throw new UnsupportedAlgorithmException(alg);
/*      */       }
/*      */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    private static byte[] DSASignaturefromDNS(byte[] dns) throws DNSSECException, IOException {
/*  584 */       if (dns.length != 41) {
/*  585 */          throw new SignatureVerificationException();
/*      */       } else {
/*  587 */          DNSInput in = new DNSInput(dns);
/*  588 */          DNSOutput out = new DNSOutput();
/*      */ 
/*  590 */          int t = in.readU8();
/*      */ 
/*  592 */          byte[] r = in.readByteArray(20);
/*  593 */          int rlen = 20;
/*  594 */          if (r[0] < 0) {
/*  595 */             ++rlen;
/*      */          }
/*  597 */          byte[] s = in.readByteArray(20);
/*  598 */          int slen = 20;
/*  599 */          if (s[0] < 0) {
/*  600 */             ++slen;
/*      */          }
/*  602 */          out.writeU8(48);
/*  603 */          out.writeU8(rlen + slen + 4);
/*      */ 
/*  605 */          out.writeU8(2);
/*  606 */          out.writeU8(rlen);
/*  607 */          if (rlen > 20) {
/*  608 */             out.writeU8(0);         }
/*  609 */          out.writeByteArray(r);
/*      */ 
/*  611 */          out.writeU8(2);
/*  612 */          out.writeU8(slen);
/*  613 */          if (slen > 20) {
/*  614 */             out.writeU8(0);         }
/*  615 */          out.writeByteArray(s);
/*      */ 
/*  617 */          return out.toByteArray();
/*      */       }
/*      */    }
/*      */ 
/*      */    private static byte[] DSASignaturetoDNS(byte[] signature, int t) throws IOException {
/*  622 */       DNSInput in = new DNSInput(signature);
/*  623 */       DNSOutput out = new DNSOutput();
/*      */ 
/*  625 */       out.writeU8(t);
/*      */ 
/*  627 */       int tmp = in.readU8();
/*  628 */       if (tmp != 48) {
/*  629 */          throw new IOException();      } else {
/*  630 */          int seqlen = in.readU8();
/*      */ 
/*  632 */          tmp = in.readU8();
/*  633 */          if (tmp != 2) {
/*  634 */             throw new IOException();         } else {
/*  635 */             int rlen = in.readU8();
/*  636 */             if (rlen == 21) {
/*  637 */                if (in.readU8() != 0) {
/*  638 */                   throw new IOException();               }
/*  639 */             } else if (rlen != 20) {
/*  640 */                throw new IOException();            }
/*  641 */             byte[] bytes = in.readByteArray(20);
/*  642 */             out.writeByteArray(bytes);
/*      */ 
/*  644 */             tmp = in.readU8();
/*  645 */             if (tmp != 2) {
/*  646 */                throw new IOException();            } else {
/*  647 */                int slen = in.readU8();
/*  648 */                if (slen == 21) {
/*  649 */                   if (in.readU8() != 0) {
/*  650 */                      throw new IOException();                  }
/*  651 */                } else if (slen != 20) {
/*  652 */                   throw new IOException();               }
/*  653 */                bytes = in.readByteArray(20);
/*  654 */                out.writeByteArray(bytes);
/*      */ 
/*  656 */                return out.toByteArray();
/*      */             }
/*      */          }
/*      */       }
/*      */    }
/*      */ 
/*      */    private static byte[] ECDSASignaturefromDNS(byte[] signature, ECKeyInfo keyinfo) throws DNSSECException, IOException {
/*  663 */       if (signature.length != keyinfo.length * 2) {
/*  664 */          throw new SignatureVerificationException();
/*      */       } else {
/*  666 */          DNSInput in = new DNSInput(signature);
/*  667 */          DNSOutput out = new DNSOutput();
/*      */ 
/*  669 */          byte[] r = in.readByteArray(keyinfo.length);
/*  670 */          int rlen = keyinfo.length;
/*  671 */          if (r[0] < 0) {
/*  672 */             ++rlen;
/*      */          }
/*  674 */          byte[] s = in.readByteArray(keyinfo.length);
/*  675 */          int slen = keyinfo.length;
/*  676 */          if (s[0] < 0) {
/*  677 */             ++slen;
/*      */          }
/*  679 */          out.writeU8(48);
/*  680 */          out.writeU8(rlen + slen + 4);
/*      */ 
/*  682 */          out.writeU8(2);
/*  683 */          out.writeU8(rlen);
/*  684 */          if (rlen > keyinfo.length) {
/*  685 */             out.writeU8(0);         }
/*  686 */          out.writeByteArray(r);
/*      */ 
/*  688 */          out.writeU8(2);
/*  689 */          out.writeU8(slen);
/*  690 */          if (slen > keyinfo.length) {
/*  691 */             out.writeU8(0);         }
/*  692 */          out.writeByteArray(s);
/*      */ 
/*  694 */          return out.toByteArray();
/*      */       }
/*      */    }
/*      */ 
/*      */    private static byte[] ECDSASignaturetoDNS(byte[] signature, ECKeyInfo keyinfo) throws IOException {
/*  699 */       DNSInput in = new DNSInput(signature);
/*  700 */       DNSOutput out = new DNSOutput();
/*      */ 
/*  702 */       int tmp = in.readU8();
/*  703 */       if (tmp != 48) {
/*  704 */          throw new IOException();      } else {
/*  705 */          int seqlen = in.readU8();
/*      */ 
/*  707 */          tmp = in.readU8();
/*  708 */          if (tmp != 2) {
/*  709 */             throw new IOException();         } else {
/*  710 */             int rlen = in.readU8();
/*  711 */             if (rlen == keyinfo.length + 1) {
/*  712 */                if (in.readU8() != 0) {
/*  713 */                   throw new IOException();               }
/*  714 */             } else if (rlen != keyinfo.length) {
/*  715 */                throw new IOException();            }
/*  716 */             byte[] bytes = in.readByteArray(keyinfo.length);
/*  717 */             out.writeByteArray(bytes);
/*      */ 
/*  719 */             tmp = in.readU8();
/*  720 */             if (tmp != 2) {
/*  721 */                throw new IOException();            } else {
/*  722 */                int slen = in.readU8();
/*  723 */                if (slen == keyinfo.length + 1) {
/*  724 */                   if (in.readU8() != 0) {
/*  725 */                      throw new IOException();                  }
/*  726 */                } else if (slen != keyinfo.length) {
/*  727 */                   throw new IOException();               }
/*  728 */                bytes = in.readByteArray(keyinfo.length);
/*  729 */                out.writeByteArray(bytes);
/*      */ 
/*  731 */                return out.toByteArray();
/*      */             }
/*      */          }
/*      */       }
/*      */    }
/*      */ 
/*      */    private static void verify(PublicKey key, int alg, byte[] data, byte[] signature) throws DNSSECException {
/*  738 */       if (key instanceof DSAPublicKey) {
/*      */          try {
/*  740 */             signature = DSASignaturefromDNS(signature);
/*      */ 
/*  742 */          } catch (IOException var7) {
/*  743 */             throw new IllegalStateException();
/*      */          }
/*  745 */       } else if (key instanceof ECPublicKey) {
/*      */          try {
/*  747 */             switch(alg) {
/*      */             case 13:
/*  749 */                signature = ECDSASignaturefromDNS(signature, ECDSA_P256);
/*      */ 
/*  751 */                break;
/*      */             case 14:
/*  753 */                signature = ECDSASignaturefromDNS(signature, ECDSA_P384);
/*      */ 
/*  755 */                break;
/*      */             default:
/*  757 */                throw new UnsupportedAlgorithmException(alg);
/*      */ 
/*      */             }
/*  760 */          } catch (IOException var6) {
/*  761 */             throw new IllegalStateException();
/*      */          }
/*      */       }
/*      */ 
/*      */       try {
/*  766 */          Signature s = Signature.getInstance(algString(alg));
/*  767 */          s.initVerify(key);
/*  768 */          s.update(data);
/*  769 */          if (!s.verify(signature)) {
/*  770 */             throw new SignatureVerificationException();
/*      */          }
/*  772 */       } catch (GeneralSecurityException var5) {
/*  773 */          throw new DNSSECException(var5.toString());
/*      */       }
/*  775 */    }
/*      */ 
/*      */ 
/*      */ 
/*      */    private static boolean matches(SIGBase sig, KEYBase key) {
/*  780 */       return key.getAlgorithm() == sig.getAlgorithm() && key.getFootprint() == sig.getFootprint() && key.getName().equals(sig.getSigner());
/*      */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    public static void verify(RRset rrset, RRSIGRecord rrsig, DNSKEYRecord key) throws DNSSECException {
/*  801 */       if (!matches(rrsig, key)) {
/*  802 */          throw new KeyMismatchException(key, rrsig);
/*      */       } else {
/*  804 */          Date now = new Date();
/*  805 */          if (now.compareTo(rrsig.getExpire()) > 0) {
/*  806 */             throw new SignatureExpiredException(rrsig.getExpire(), now);
/*  807 */          } else if (now.compareTo(rrsig.getTimeSigned()) < 0) {
/*  808 */             throw new SignatureNotYetValidException(rrsig.getTimeSigned(), now);
/*      */ 
/*      */          } else {
/*  811 */             verify(key.getPublicKey(), rrsig.getAlgorithm(), digestRRset(rrsig, rrset), rrsig.getSignature());
/*      */          }      }
/*  813 */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    private static byte[] sign(PrivateKey privkey, PublicKey pubkey, int alg, byte[] data, String provider) throws DNSSECException {
/*      */       byte[] signature;
/*      */       try {
/*      */          Signature s;
/*  822 */          if (provider != null) {
/*  823 */             s = Signature.getInstance(algString(alg), provider);
/*      */          } else {
/*  825 */             s = Signature.getInstance(algString(alg));         }
/*  826 */          s.initSign(privkey);
/*  827 */          s.update(data);
/*  828 */          signature = s.sign();
/*      */ 
/*  830 */       } catch (GeneralSecurityException var11) {
/*  831 */          throw new DNSSECException(var11.toString());
/*      */       }
/*      */ 
/*  834 */       if (pubkey instanceof DSAPublicKey) {
/*      */          try {
/*  836 */             DSAPublicKey dsa = (DSAPublicKey)pubkey;
/*  837 */             BigInteger P = dsa.getParams().getP();
/*  838 */             int t = (BigIntegerLength(P) - 64) / 8;
/*  839 */             signature = DSASignaturetoDNS(signature, t);
/*      */ 
/*  841 */          } catch (IOException var10) {
/*  842 */             throw new IllegalStateException();
/*      */          }
/*  844 */       } else if (pubkey instanceof ECPublicKey) {
/*      */          try {
/*  846 */             switch(alg) {
/*      */             case 13:
/*  848 */                signature = ECDSASignaturetoDNS(signature, ECDSA_P256);
/*      */ 
/*  850 */                break;
/*      */             case 14:
/*  852 */                signature = ECDSASignaturetoDNS(signature, ECDSA_P384);
/*      */ 
/*  854 */                break;
/*      */             default:
/*  856 */                throw new UnsupportedAlgorithmException(alg);
/*      */ 
/*      */             }
/*  859 */          } catch (IOException var9) {
/*  860 */             throw new IllegalStateException();
/*      */          }
/*      */       }
/*      */ 
/*  864 */       return signature;
/*      */    }
/*      */ 
/*      */ 
/*      */    static void checkAlgorithm(PrivateKey key, int alg) throws UnsupportedAlgorithmException {
/*  869 */       switch(alg) {
/*      */       case 1:
/*      */       case 5:
/*      */       case 7:
/*      */       case 8:
/*      */       case 10:
/*  875 */          if (!(key instanceof RSAPrivateKey)) {
/*  876 */             throw new IncompatibleKeyException();
/*      */ 
/*      */       case 3:
/*      */       case 6:
/*  880 */          if (!(key instanceof DSAPrivateKey)) {         }
/*  881 */             throw new IncompatibleKeyException();
/*      */          break;
/*      */       case 2:
/*      */       case 4:         }         break;      case 13:      case 14:      case 9:
/*  885 */          if (!(key instanceof ECPrivateKey)) {      case 11:
/*  886 */             throw new IncompatibleKeyException();
/*      */       case 12:
/*      */       default:
/*  889 */          throw new UnsupportedAlgorithmException(alg);
/*      */          }      }
/*  891 */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    public static RRSIGRecord sign(RRset rrset, DNSKEYRecord key, PrivateKey privkey, Date inception, Date expiration) throws DNSSECException {
/*  910 */       return sign(rrset, key, privkey, inception, expiration, (String)null);
/*      */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    public static RRSIGRecord sign(RRset rrset, DNSKEYRecord key, PrivateKey privkey, Date inception, Date expiration, String provider) throws DNSSECException {
/*  932 */       int alg = key.getAlgorithm();
/*  933 */       checkAlgorithm(privkey, alg);
/*      */ 
/*  935 */       RRSIGRecord rrsig = new RRSIGRecord(rrset.getName(), rrset.getDClass(), rrset.getTTL(), rrset.getType(), alg, rrset.getTTL(), expiration, inception, key.getFootprint(), key.getName(), (byte[])null);
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*  942 */       rrsig.setSignature(sign(privkey, key.getPublicKey(), alg, digestRRset(rrsig, rrset), provider));
/*      */ 
/*  944 */       return rrsig;
/*      */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    static SIGRecord signMessage(Message message, SIGRecord previous, KEYRecord key, PrivateKey privkey, Date inception, Date expiration) throws DNSSECException {
/*  952 */       int alg = key.getAlgorithm();
/*  953 */       checkAlgorithm(privkey, alg);
/*      */ 
/*  955 */       SIGRecord sig = new SIGRecord(Name.root, 255, 0L, 0, alg, 0L, expiration, inception, key.getFootprint(), key.getName(), (byte[])null);
/*      */ 
/*      */ 
/*      */ 
/*  959 */       DNSOutput out = new DNSOutput();
/*  960 */       digestSIG(out, sig);
/*  961 */       if (previous != null) {
/*  962 */          out.writeByteArray(previous.getSignature());      }
/*  963 */       out.writeByteArray(message.toWire());
/*      */ 
/*  965 */       sig.setSignature(sign(privkey, key.getPublicKey(), alg, out.toByteArray(), (String)null));
/*      */ 
/*  967 */       return sig;
/*      */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    static void verifyMessage(Message message, byte[] bytes, SIGRecord sig, SIGRecord previous, KEYRecord key) throws DNSSECException {
/*  974 */       if (message.sig0start == 0) {
/*  975 */          throw new NoSignatureException();
/*      */ 
/*  977 */       } else if (!matches(sig, key)) {
/*  978 */          throw new KeyMismatchException(key, sig);
/*      */       } else {
/*  980 */          Date now = new Date();
/*      */ 
/*  982 */          if (now.compareTo(sig.getExpire()) > 0) {
/*  983 */             throw new SignatureExpiredException(sig.getExpire(), now);
/*  984 */          } else if (now.compareTo(sig.getTimeSigned()) < 0) {
/*  985 */             throw new SignatureNotYetValidException(sig.getTimeSigned(), now);
/*      */ 
/*      */          } else {
/*  988 */             DNSOutput out = new DNSOutput();
/*  989 */             digestSIG(out, sig);
/*  990 */             if (previous != null) {
/*  991 */                out.writeByteArray(previous.getSignature());
/*      */             }
/*  993 */             Header header = (Header)message.getHeader().clone();
/*  994 */             header.decCount(3);
/*  995 */             out.writeByteArray(header.toWire());
/*      */ 
/*  997 */             out.writeByteArray(bytes, 12, message.sig0start - 12);
/*      */ 
/*      */ 
/* 1000 */             verify(key.getPublicKey(), sig.getAlgorithm(), out.toByteArray(), sig.getSignature());
/*      */          }      }
/* 1002 */    }
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */ 
/*      */    static byte[] generateDSDigest(DNSKEYRecord key, int digestid) {
/*      */       MessageDigest digest;
/*      */       try {
/* 1015 */          switch(digestid) {
/*      */          case 1:
/* 1017 */             digest = MessageDigest.getInstance("sha-1");
/* 1018 */             break;
/*      */          case 2:
/* 1020 */             digest = MessageDigest.getInstance("sha-256");
/* 1021 */             break;
/*      */          case 4:
/* 1023 */             digest = MessageDigest.getInstance("sha-384");
/*      */          case 3:
/*      */          default:
/* 1026 */             throw new IllegalArgumentException("unknown DS digest type " + digestid);
/*      */ 
/*      */ 
/*      */          }
/* 1030 */       } catch (NoSuchAlgorithmException var4) {
/* 1031 */          throw new IllegalStateException("no message digest support");
/*      */       }
/* 1033 */       digest.update(key.getName().toWireCanonical());
/* 1034 */       digest.update(key.rdataToWireCanonical());
/* 1035 */       return digest.digest();
/*      */    }
/*      */ }
