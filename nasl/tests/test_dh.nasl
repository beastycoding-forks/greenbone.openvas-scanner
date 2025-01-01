# SPDX-FileCopyrightText: 2023-2025 Greenbone AG
# Some text descriptions might be excerpted from (a) referenced
# source(s), and are Copyright (C) by the respective right holder(s).
#
# SPDX-License-Identifier: GPL-2.0-or-later

# OpenVAS Testsuite for the NASL interpreter
# Description: Tests for the nasl functions dh_generate_key and dh_compute_key

prime = raw_string(0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
		   0xC9, 0x0F, 0xDA, 0xA2, 0x21, 0x68, 0xC2, 0x34,
		   0xC4, 0xC6, 0x62, 0x8B, 0x80, 0xDC, 0x1C, 0xD1,
		   0x29, 0x02, 0x4E, 0x08, 0x8A, 0x67, 0xCC, 0x74,
		   0x02, 0x0B, 0xBE, 0xA6, 0x3B, 0x13, 0x9B, 0x22,
		   0x51, 0x4A, 0x08, 0x79, 0x8E, 0x34, 0x04, 0xDD,
		   0xEF, 0x95, 0x19, 0xB3, 0xCD, 0x3A, 0x43, 0x1B,
		   0x30, 0x2B, 0x0A, 0x6D, 0xF2, 0x5F, 0x14, 0x37,
		   0x4F, 0xE1, 0x35, 0x6D, 0x6D, 0x51, 0xC2, 0x45,
		   0xE4, 0x85, 0xB5, 0x76, 0x62, 0x5E, 0x7E, 0xC6,
		   0xF4, 0x4C, 0x42, 0xE9, 0xA6, 0x37, 0xED, 0x6B,
		   0x0B, 0xFF, 0x5C, 0xB6, 0xF4, 0x06, 0xB7, 0xED,
		   0xEE, 0x38, 0x6B, 0xFB, 0x5A, 0x89, 0x9F, 0xA5,
		   0xAE, 0x9F, 0x24, 0x11, 0x7C, 0x4B, 0x1F, 0xE6,
		   0x49, 0x28, 0x66, 0x51, 0xEC, 0xE6, 0x53, 0x81,
		   0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF);

generator = raw_string(2);


function test_dh_generate_key()
{
  local_var priv, pub, expected;

  testcase_start("test_dh_generate_key");

  # normally, this is a random string. For the test we use a fixed
  # string to get predictable results.
  priv = raw_string(0x51, 0x9a, 0xed, 0x06, 0xe4, 0x85, 0x1d, 0xe1,
		    0x29, 0x7c, 0x57, 0xee, 0xbc, 0xf2, 0x14, 0x03,
		    0x65, 0x86, 0x50, 0x9c, 0xdb, 0x6d, 0x35, 0xdc,
		    0x0f, 0x32, 0x71, 0xff, 0xae, 0xe6, 0x1c, 0xc8,
		    0xab, 0xa1, 0x92, 0x0b, 0xc9, 0xd4, 0xf8, 0x39);

  pub = dh_generate_key(p:prime, g:generator, priv:priv);

  expected = strcat("4979724ac0e28486dc6183f4acafa4c9",
		    "fc86ba515449fc23e411414865543197",
		    "6588771edfa0e00cede9b6a095a57b09",
		    "4e53fccc99737cdfd217be9186fa6a44",
		    "a4a8fc48227f6512ce8df0df87d23b81",
		    "14fa663c9e06f86c2be9d74ce59ceb61",
		    "23c0df1c7d77272aef9163c5ba170e4b",
		    "ddc150a86dfea26d864556c449450607");

  if (expected == hexstr(pub))
    testcase_ok();
  else
    testcase_failed();
}


test_dh_generate_key();



function test_dh_compute_key()
{
  local_var client_pub, client_priv, server_pub, server_priv, shared, expected;

  testcase_start("test_dh_compute_key");

  # normally, this is a random string. For the test we use a fixed
  # string to get predictable results.
  server_priv = raw_string(0x00, 0x86, 0x14, 0x2d, 0xa9, 0xa3, 0x73, 0x46,
			   0x3f, 0x89, 0x1c, 0x6d, 0xd3, 0x09, 0xe9, 0xfb,
			   0x2e, 0x16, 0x52, 0x67, 0x59, 0xdb, 0x80, 0x22,
			   0x8e, 0xab, 0x42, 0xe8, 0x21, 0x90, 0xcd, 0x78,
			   0xb7, 0x7f, 0x3b, 0x8a, 0xf4, 0x27, 0x92, 0xd9,
			   0xd2);
  server_pub = dh_generate_key(p:prime, g:generator, priv:server_priv);

  client_priv = raw_string(0x51, 0x9a, 0xed, 0x06, 0xe4, 0x85, 0x1d, 0xe1,
			   0x29, 0x7c, 0x57, 0xee, 0xbc, 0xf2, 0x14, 0x03,
			   0x65, 0x86, 0x50, 0x9c, 0xdb, 0x6d, 0x35, 0xdc,
			   0x0f, 0x32, 0x71, 0xff, 0xae, 0xe6, 0x1c, 0xc8,
			   0xab, 0xa1, 0x92, 0x0b, 0xc9, 0xd4, 0xf8, 0x39);

  client_pub = dh_generate_key(p:prime, g:generator, priv:client_priv);

  shared = dh_compute_key(p:prime, g:generator, dh_server_pub:server_pub,
			  pub_key:client_pub, priv_key:client_priv);

  expected = strcat("677ce52a2a803bca17443b2e69a8add3",
		    "cc7c069eaea90ab7ed90af04d684ad55",
		    "9443f8db8163d4e112e91b2520bf138e",
		    "94863dacfc85efc5363a392540b5e070",
		    "a5c75da2119df9ab1c3219384ee37c32",
		    "d9f2deda3afba03571c74c987514e0e1",
		    "10f05678da23518d6d6359a8a74f2320",
		    "cab036b416ee4989fdd5a39ae40040d7");

  if (expected == hexstr(shared))
    testcase_ok();
  else
    {
      testcase_failed();
      display("shared=", hexstr(shared), "\n");
    }
}


test_dh_compute_key();
