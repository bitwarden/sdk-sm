package com.bitwarden.sdk;

import com.sun.jna.Library;
import com.sun.jna.Pointer;

public interface BitwardenLibrary extends Library {

    Pointer init(String clientSettings);

    void free_mem(Pointer client);

    Pointer run_command(String command, Pointer client);

    void free_string(Pointer str);
}
