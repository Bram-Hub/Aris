package edu.rpi.aris.assign.server.auth;

import org.apache.commons.io.IOUtils;
import org.apache.commons.lang3.SystemUtils;
import org.apache.logging.log4j.LogManager;
import org.apache.logging.log4j.Logger;

import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;

public class PAMLoginAuth extends LoginAuth {

    private static final String LIB_NAME = "libassign_pam";
    private static final Logger log = LogManager.getLogger();
    private static final PAMLoginAuth instance = new PAMLoginAuth();
    private static boolean loaded = false;

    static {
        loadLib();
    }

    private PAMLoginAuth() {
    }

    private static void loadLib() {
        if (!SystemUtils.IS_OS_LINUX) {
            log.info("PAM authentication is only available on linux and has been disabled");
            return;
        }
        log.info("Loading libassign_pam.so native library");

        File tmpFile = new File(System.getProperty("java.io.tmpdir"), LIB_NAME + ".so");
        int i = 0;
        while (tmpFile.exists() && !tmpFile.delete())
            tmpFile = new File(System.getProperty("java.io.tmpdir"), LIB_NAME + (i++) + ".so");
        boolean copied = false;
        try (InputStream in = ClassLoader.getSystemResourceAsStream(LIB_NAME + ".so");
             FileOutputStream out = new FileOutputStream(tmpFile)) {
            if (in != null) {
                IOUtils.copy(in, out);
                copied = true;
            }
        } catch (IOException e) {
            copied = false;
            log.error("Failed to extract libassign_pam to temp directory", e);
        }
        if (copied) {
            try {
                System.load(tmpFile.getCanonicalPath());
                loaded = true;
            } catch (Exception e) {
                log.error("Failed to load native libassign_pam library", e);
            }
        }
    }

    static PAMLoginAuth getInstance() {
        return instance;
    }

    private static native PAMResponse pam_authenticate(String user, String pass);

    @Override
    protected String checkAuth(String user, String pass, String salt, String savedHash) {
        PAMResponse response = pam_authenticate(user, pass);
        if (response == null)
            return "PAM library returned no response";
        if (response.success)
            return null;
        return response.error == null ? "PAM encountered an unknown error" : response.error;
    }

    @Override
    public boolean isSupported() {
        return loaded;
    }

}