package edu.rpi.aris.net;

public class NetUtil {

    public static final String ARIS_NAME = "Aris-Java";
    public static final String AUTH_BAN = "AUTH BAN";
    public static final String AUTH_FAIL = "AUTH FAIL";
    public static final String AUTH_OK = "AUTH OK";
    public static final String AUTH_ERR = "AUTH ERROR";
    public static final String AUTH_INVALID = "AUTH INVALID";
    public static final String AUTH = "AUTH";
    public static final String AUTH_PASS = "PASS";
    public static final String AUTH_ACCESS_TOKEN = "TOKEN";

    public static final String GET_USER_TYPE = "GET_USER_TYPE";
    public static final String GET_ASSIGNMENTS = "GET_ASSIGNMENTS";
    public static final String GET_PROOFS = "GET_PROOFS";
    public static final String LIST_SUBMISSIONS = "LIST_SUBMISSION";
    public static final String GET_SUBMISSION = "GET_SUBMISSION";
    public static final String CREATE_SUBMISSION = "CREATE_SUBMISSION";
    public static final String CREATE_ASSIGNMENT = "CREATE_ASSIGNMENT";
    public static final String DELETE_ASSIGNMENT = "DELETE_ASSIGNMENT";
    public static final String UPDATE_ASSIGNMENT = "UPDATE_ASSIGNMENT";
    public static final String CREATE_PROOF = "CREATE_PROOF";
    public static final String DELETE_PROOF = "DELETE_PROOF";
    public static final String UPDATE_PROOF = "UPDATE_PROOF";
    public static final String CREATE_USER = "CREATE_USER";
    public static final String DELETE_USER = "DELETE_USER";
    public static final String UPDATE_USER = "UPDATE_USER";
    public static final String CREATE_CLASS = "CREATE_CLASS";
    public static final String DELETE_CLASS = "DELETE_CLASS";
    public static final String UPDATE_CLASS = "UPDATE_CLASS";

    public static final String USER_STUDENT = "student";
    public static final String USER_INSTRUCTOR = "instructor";

    public static final long MAX_FILE_SIZE = 5242880; // 5MiB
    public static final String UNAUTHORIZED = "UNAUTHORIZED";
    public static final String ERROR = "ERROR";
    public static final String OK = "OK";
    public static final String INVALID = "INVALID";

    /**
     * Compares two version strings.
     * <p>
     * Use this instead of String.compareTo() for a non-lexicographical
     * comparison that works for version strings. e.g. "1.10".compareTo("1.6").
     * <p>
     * <p>
     * <p>
     * Note: It does not work if "1.10" is supposed to be equal to "1.10.0".
     *
     * @param str1 a string of ordinal numbers separated by decimal points.
     * @param str2 a string of ordinal numbers separated by decimal points.
     * @return The result is a negative integer if str1 is numerically less than str2.
     * The result is a positive integer if str1 is numerically greater than str2.
     * The result is zero if the strings are numerically equal.
     */
    public static int versionCompare(String str1, String str2) {
        String[] vals1 = str1.split("\\.");
        String[] vals2 = str2.split("\\.");
        int i = 0;
        // set index to first non-equal ordinal or length of shortest version string
        while (i < vals1.length && i < vals2.length && vals1[i].equals(vals2[i])) {
            i++;
        }
        // compare first non-equal ordinal number
        if (i < vals1.length && i < vals2.length) {
            int diff = Integer.valueOf(vals1[i]).compareTo(Integer.valueOf(vals2[i]));
            return Integer.signum(diff);
        }
        // the strings are equal or one string is a substring of the other
        // e.g. "1.2.3" = "1.2.3" or "1.2.3" < "1.2.3.4"
        return Integer.signum(vals1.length - vals2.length);
    }

}