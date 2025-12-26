#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

#define MAX_DEFINES 1024
#define MAX_INCLUDE_DEPTH 16
#define BUFFER_SIZE 4096

typedef struct {
    char *name;
    char *value;
} Define;

Define defines[MAX_DEFINES];
int num_defines = 0;

FILE *include_stack[MAX_INCLUDE_DEPTH];
char *include_filenames[MAX_INCLUDE_DEPTH];
int include_depth = 0;

char *include_paths[] = {"/usr/include", ".", NULL}; // Example paths

int is_whitespace(char c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r';
}

void trim(char *str) {
    char *end;
    while (is_whitespace(*str)) str++;
    if (*str == 0) return;
    end = str + strlen(str) - 1;
    while (end > str && is_whitespace(*end)) end--;
    end[1] = '\0';
}

char *find_define(const char *name) {
    for (int i = 0; i < num_defines; i++) {
        if (strcmp(defines[i].name, name) == 0) {
            return defines[i].value;
        }
    }
    return NULL;
}

void add_define(const char *name, const char *value) {
    if (num_defines >= MAX_DEFINES) {
        fprintf(stderr, "Too many defines\n");
        exit(1);
    }
    defines[num_defines].name = strdup(name);
    defines[num_defines].value = strdup(value ? value : "");
    num_defines++;
}

void remove_define(const char *name) {
    for (int i = 0; i < num_defines; i++) {
        if (strcmp(defines[i].name, name) == 0) {
            free(defines[i].name);
            free(defines[i].value);
            defines[i] = defines[--num_defines];
            return;
        }
    }
}

FILE *open_include(const char *filename, int system) {
    FILE *fp = NULL;
    if (system) {
        for (char **path = include_paths; *path; path++) {
            char fullpath[BUFFER_SIZE];
            snprintf(fullpath, sizeof(fullpath), "%s/%s", *path, filename);
            fp = fopen(fullpath, "r");
            if (fp) return fp;
        }
    } else {
        fp = fopen(filename, "r");
    }
    return fp;
}

void process_directive(char *line, FILE *output) {
    char *directive = line + 1;
    while (is_whitespace(*directive)) directive++;

    if (strncmp(directive, "include", 7) == 0) {
        directive += 7;
        while (is_whitespace(*directive)) directive++;
        int system = (*directive == '<');
        char *filename = directive + 1;
        char *end = strchr(filename, system ? '>' : '"');
        if (!end) {
            fprintf(stderr, "Invalid include\n");
            exit(1);
        }
        *end = '\0';
        FILE *fp = open_include(filename, system);
        if (!fp) {
            fprintf(stderr, "Cannot open include: %s\n", filename);
            exit(1);
        }
        if (include_depth >= MAX_INCLUDE_DEPTH) {
            fprintf(stderr, "Include depth exceeded\n");
            exit(1);
        }
        include_stack[include_depth] = fp;
        include_filenames[include_depth] = strdup(filename);
        include_depth++;
    } else if (strncmp(directive, "define", 6) == 0) {
        directive += 6;
        while (is_whitespace(*directive)) directive++;
        char *name = directive;
        while (*directive && !is_whitespace(*directive)) directive++;
        if (*directive) *directive++ = '\0';
        while (is_whitespace(*directive)) directive++;
        char *value = directive;
        trim(value);
        add_define(name, value);
    } else if (strncmp(directive, "undef", 5) == 0) {
        directive += 5;
        while (is_whitespace(*directive)) directive++;
        char *name = directive;
        trim(name);
        remove_define(name);
    } else if (strncmp(directive, "ifdef", 5) == 0 || strncmp(directive, "ifndef", 6) == 0) {
        // Simplified: skip for now
        fprintf(output, "%s\n", line);
    } else {
        // Other directives: pass through or error
        fprintf(output, "%s\n", line);
    }
}

void expand_macros(char *line, FILE *output) {
    char buffer[BUFFER_SIZE * 2];
    char *out = buffer;
    char *in = line;
    while (*in) {
        if (isalpha(*in) || *in == '_') {
            char *start = in;
            while (isalnum(*in) || *in == '_') in++;
            char temp = *in;
            *in = '\0';
            char *value = find_define(start);
            *in = temp;
            if (value) {
                strcpy(out, value);
                out += strlen(value);
            } else {
                strcpy(out, start);
                out += strlen(start);
            }
        } else {
            *out++ = *in++;
        }
    }
    *out = '\0';
    fprintf(output, "%s\n", buffer);
}

void preprocess(FILE *input, FILE *output, const char *filename) {
    char line[BUFFER_SIZE];
    while (1) {
        if (fgets(line, sizeof(line), input) == NULL) {
            if (include_depth > 0) {
                fclose(include_stack[--include_depth]);
                free(include_filenames[include_depth]);
                input = include_depth > 0 ? include_stack[include_depth - 1] : stdin; // Assuming main input is stdin for simplicity
                continue;
            }
            break;
        }
        char *trimmed = line;
        while (is_whitespace(*trimmed)) trimmed++;
        if (*trimmed == '#') {
            process_directive(trimmed, output);
        } else {
            expand_macros(line, output);
        }
    }
}

int main(int argc, char *argv[]) {
    if (argc < 3) {
        fprintf(stderr, "Usage: preprocessor input.vira output.c\n");
        return 1;
    }

    FILE *input = fopen(argv[1], "r");
    if (!input) {
        fprintf(stderr, "Cannot open input: %s\n", argv[1]);
        return 1;
    }

    FILE *output = fopen(argv[2], "w");
    if (!output) {
        fprintf(stderr, "Cannot open output: %s\n", argv[2]);
        fclose(input);
        return 1;
    }

    include_stack[0] = input;
    include_filenames[0] = strdup(argv[1]);
    include_depth = 1;

    preprocess(input, output, argv[1]);

    fclose(output);
    // Note: input closed in preprocess

    for (int i = 0; i < num_defines; i++) {
        free(defines[i].name);
        free(defines[i].value);
    }

    return 0;
}
