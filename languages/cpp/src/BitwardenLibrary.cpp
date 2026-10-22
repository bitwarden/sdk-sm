#include "BitwardenLibrary.h"
#include <iostream>

BitwardenLibrary::BitwardenLibrary(const std::string& providedLibraryPath) : libraryHandle(nullptr) {
    std::string libraryExtension;
    std::string libraryNameUnix = "libbitwarden_c";
    std::string libraryNameWin = "bitwarden_c";
#if defined(_WIN32)
    libraryExtension = ".dll";
#elif defined(__linux__)
    libraryExtension = ".so";
#elif defined(__APPLE__)
    libraryExtension = ".dylib";
#else
    // Unsupported platform
    std::cerr << "Unsupported platform." << std::endl;
    return;
#endif

    // Load the dynamic library
#ifdef _WIN32
    std::string libraryPath = providedLibraryPath + libraryNameWin + libraryExtension;
    // Load the dynamic library on Windows
    libraryHandle = LoadLibraryA(libraryPath.c_str());

    if (!libraryHandle) {
        std::cerr << "Failed to load the Bitwarden library." << std::endl;
    }
#else
    std::string libraryPath = providedLibraryPath + libraryNameUnix + libraryExtension;
    // Load the dynamic library on Unix-based systems (Linux, macOS)
    libraryHandle = dlopen(libraryPath.c_str(), RTLD_NOW);

    if (!libraryHandle) {
        std::cerr << "Failed to load the Bitwarden library: " << dlerror() << std::endl;
    }
#endif
}

BitwardenLibrary::~BitwardenLibrary() {
    if (libraryHandle) {
#ifdef _WIN32
        FreeLibrary(libraryHandle);
#else
        dlclose(libraryHandle);
#endif
    }
}

void* BitwardenLibrary::init(const char* clientSettingsJson) {
    typedef void* (*InitFunction)(const char*);
    InitFunction initFunction = nullptr;

#ifdef _WIN32
    // Get the address of the init function on Windows
    initFunction = reinterpret_cast<InitFunction>(GetProcAddress(libraryHandle, "init"));
#else
    // Get the address of the init function on Unix-based systems
    initFunction = reinterpret_cast<InitFunction>(dlsym(libraryHandle, "init"));
#endif

    if (initFunction) {
        return initFunction(clientSettingsJson);
    }

    std::cerr << "Failed to load init function from the Bitwarden library: " << std::endl;
    return nullptr;
}

void BitwardenLibrary::free_mem(void* client) {
    typedef void (*FreeMemFunction)(void*);
    FreeMemFunction freeMemFunction = nullptr;

#ifdef _WIN32
    // Get the address of the free_mem function on Windows
    freeMemFunction = reinterpret_cast<FreeMemFunction>(GetProcAddress(libraryHandle, "free_mem"));
#else
    // Get the address of the free_mem function on Unix-based systems
    freeMemFunction = reinterpret_cast<FreeMemFunction>(dlsym(libraryHandle, "free_mem"));
#endif

    if (freeMemFunction) {
        freeMemFunction(client);
    } else {
        std::cerr << "Failed to load free_mem function from the Bitwarden library." << std::endl;
    }
}

const char* BitwardenLibrary::run_command(const char* commandJson, void* client) {
    typedef const char* (*RunCommandFunction)(const char*, void*);
    RunCommandFunction runCommandFunction = nullptr;

#ifdef _WIN32
    // Get the address of the run_command function on Windows
    runCommandFunction = reinterpret_cast<RunCommandFunction>(GetProcAddress(libraryHandle, "run_command"));
#else
    // Get the address of the run_command function on Unix-based systems
    runCommandFunction = reinterpret_cast<RunCommandFunction>(dlsym(libraryHandle, "run_command"));
#endif

    if (runCommandFunction) {
        return runCommandFunction(commandJson, client);
    }

    std::cerr << "Failed to load run_command function from the Bitwarden library." << std::endl;
    return nullptr;
}

void BitwardenLibrary::free_string(const char* str) {
    typedef void (*FreeStringFunction)(const char*);
    FreeStringFunction freeStringFunction = nullptr;

#ifdef _WIN32
    freeStringFunction = reinterpret_cast<FreeStringFunction>(GetProcAddress(libraryHandle, "free_string"));
#else
    freeStringFunction = reinterpret_cast<FreeStringFunction>(dlsym(libraryHandle, "free_string"));
#endif

    if (freeStringFunction) {
        freeStringFunction(str);
    } else {
        std::cerr << "Failed to load free_string function from the Bitwarden library." << std::endl;
    }
}
