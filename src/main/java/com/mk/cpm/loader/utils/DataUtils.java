package com.mk.cpm.loader.utils;

import java.io.*;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class DataUtils {

    public String getValues(List<String> infos, File file, String parameter) {
        if (file == null) return null;
        if (parameter == null) return null;
        if (parameter.isEmpty()) return null;
        BufferedReader br = null;
        try {
            br = new BufferedReader(new FileReader(file));
        } catch (FileNotFoundException e) {
            throw new RuntimeException(e);
        }
        List<String> lines = new ArrayList<>();
        try {
            String line;
            while ((line = br.readLine()) != null) {
                lines.add(line);
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
        try {
            for (String line : lines) {
                if (line.contains(parameter)) {
                    if (infos == null) {
                        new ArrayList<>();
                    }
                    if (line.split(":").length < 2) {
                        return null;
                    }
                    String temp = parameter + ": " + line.split(":")[1].trim();
                    infos.add(temp);
                    return line.split(":")[1].trim();
                }
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
        return null;
    }

    //load section and set this.lines
    /*
    public Object load(File file, String section){
        if (file == null) return;
        if (section == null) return;
        if (section.isEmpty()) return;
        BufferedReader br = null;
        try {
            br = new BufferedReader(new FileReader(file));
        } catch (FileNotFoundException e) {
            throw new RuntimeException(e);
        }
        lines = new ArrayList<>();
        try {
            String line;
            while ((line = br.readLine()) != null) {
                lines.add(line);
            }
        } catch (Exception e) {
            e.printStackTrace();
        }

    }

     */

    public String getValuesFromSection(List<String> infos, File file, String section, String parametre) {

        if (section == null) return null;
        if (parametre.isEmpty()) return null;
        BufferedReader br = null;
        try {
            br = new BufferedReader(new FileReader(file));
        } catch (FileNotFoundException e) {
            throw new RuntimeException(e);
        }
        List<String> lines = new ArrayList<>();
        try {
            String line;
            Boolean sect = false;
            while ((line = br.readLine()) != null) {
                //check if section is found
                if (line.contains(section)) {
                    sect = true;
                }
                if (line.contains("}") && sect) {
                    sect = false;
                }
                if (sect) {
                    lines.add(line);
                }
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
        try {
            for (String line : lines) {
                if (line.contains(parametre)) {
                    if (infos == null) {
                        new ArrayList<>();
                    }
                    String temp = parametre.replace(":", "") + ": " + line.split(":")[1].trim();
                    infos.add(temp);
                    return line.split(":")[1].trim();
                }
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
        return null;

    }

    public List<String> getAllSections(File file) {
        if (file == null) return null;
        BufferedReader br = null;
        try {
            br = new BufferedReader(new FileReader(file));
        } catch (FileNotFoundException e) {
            throw new RuntimeException(e);
        }
        List<String> lines = new ArrayList<>();
        try {
            String line;
            while ((line = br.readLine()) != null) {
                if (line.contains("{")) {
                    lines.add(line);
                }
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
        return lines;
    }

    public HashMap<String, String> getAllValuesOfSection(File file, String sectionName) {
        if (file == null) return null;
        if (sectionName == null) return null;
        if (sectionName.isEmpty()) return null;
        BufferedReader br = null;
        try {
            br = new BufferedReader(new FileReader(file));
        } catch (FileNotFoundException e) {
            throw new RuntimeException(e);
        }
        HashMap<String, String> values = new HashMap<>();
        try {
            String line;
            Boolean sect = false;
            while ((line = br.readLine()) != null) {
                //check if section is found
                if (line.contains(sectionName)) {
                    sect = true;
                }
                if (line.contains("}") && sect) {
                    sect = false;
                }
                if (sect) {
                    if (line.contains(":")) {
                        values.put(line.split(":")[0].trim(), line.split(":")[1].trim());
                    }
                }
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
        return values;
    }


    public void setValues(File file, String parameter, String value) {
        if (file == null) return;
        if (parameter == null) return;
        if (value == null) return;
        if (parameter.isEmpty()) return;
        if (value.isEmpty()) return;
        FileWriter fileWriter = null;
        try {
            fileWriter = new FileWriter(file, true);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        try {
            fileWriter.write(parameter + ": " + value + "\n");
            fileWriter.flush();
            fileWriter.close();
        } catch (Exception e) {
            System.out.println("Error in setValues");
            System.out.println("File " + file);
            System.out.println("Parameter " + parameter);
            System.out.println("Value " + value);
        }

    }


    public void setMultiValues(FileWriter file, String section, Map<String, String> values) {
        if (file == null) return;
        if (section == null) return;
        if (values == null) return;
        if (section.isEmpty()) return;
        if (values.isEmpty()) return;
        try {
            file.write(section + "\n");
            for (Map.Entry<String, String> entry : values.entrySet()) {
                if (entry.getKey() == null) {
                    continue;
                }
                if (entry.getValue() == null) {
                    continue;
                }
                if (entry.getKey().isEmpty()) {
                    continue;
                }
                if (entry.getValue().isEmpty()) {
                    continue;
                }
                file.write(entry.getKey() + ": " + entry.getValue() + "\n");
            }
        } catch (Exception e) {
            System.out.println("Error in setMultiValues");
            System.out.println("File " + file);
            System.out.println("Section " + section);
            System.out.println("Values " + values);
        }
    }

    public void setMultiValues(File file, String section, Map<String, String> values) {
        if (file == null) return;
        if (section == null) return;
        if (values == null) return;
        if (section.isEmpty()) return;
        if (values.isEmpty()) return;
        FileWriter fileWriter = null;
        try {
            fileWriter = new FileWriter(file, true);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
        try {
            fileWriter.write("\n");
            fileWriter.write(section + "\n");
            for (Map.Entry<String, String> entry : values.entrySet()) {
                if (entry.getKey() == null) {
                    continue;
                }
                if (entry.getValue() == null) {
                    continue;
                }
                if (entry.getKey().isEmpty()) {
                    continue;
                }
                if (entry.getValue().isEmpty()) {
                    continue;
                }
                fileWriter.write("  " + entry.getKey() + ": " + entry.getValue() + "\n");
            }
            fileWriter.write("}\n");
            fileWriter.flush();
            fileWriter.close();
        } catch (Exception e) {
            System.out.println("Error in setMultiValues");
            System.out.println("File " + file);
            System.out.println("Section " + section);
            System.out.println("Values " + values);
        }
    }


    //get all name of section with name
    public List<String> getSections(File file, String name) {
        if (file == null) return null;
        if (name == null) return null;
        if (name.isEmpty()) return null;
        BufferedReader br = null;
        try {
            br = new BufferedReader(new FileReader(file));
        } catch (FileNotFoundException e) {
            throw new RuntimeException(e);
        }
        List<String> lines = new ArrayList<>();
        try {
            String line;
            while ((line = br.readLine()) != null) {
                if (line.contains(name) && line.contains("{")) {
                    lines.add(line);
                }
            }
        } catch (Exception e) {
            e.printStackTrace();
        }
        return lines;
    }
}
