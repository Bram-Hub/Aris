package edu.rpi.aris.net.message;

import edu.rpi.aris.net.NetUtil;
import edu.rpi.aris.net.User;

import java.sql.Connection;
import java.sql.PreparedStatement;
import java.sql.SQLException;

public class ClassDeleteMsg extends Message {

    private final int cid;

    public ClassDeleteMsg(int cid) {
        this.cid = cid;
    }

    // DO NOT REMOVE!! Default constructor is required for gson deserialization
    private ClassDeleteMsg() {
        cid = 0;
    }

    @Override
    public ErrorType processMessage(Connection connection, User user) throws SQLException {
        if (!user.userType.equals(NetUtil.USER_INSTRUCTOR))
            return ErrorType.UNAUTHORIZED;
        try (PreparedStatement deleteClass = connection.prepareStatement("DELETE FROM class WHERE id = ?;")) {
            deleteClass.setInt(1, cid);
            deleteClass.executeUpdate();
        }
        return null;
    }

    @Override
    public MessageType getMessageType() {
        return MessageType.DELETE_CLASS;
    }

    @Override
    public boolean checkValid() {
        return cid > 0;
    }
}