package com.mk.cpm.converter;

import com.mk.cpm.config.AddonSettingConfig;
import javassist.ClassPool;
import javassist.CtClass;
import javassist.bytecode.AnnotationsAttribute;
import javassist.bytecode.ClassFile;
import javassist.bytecode.FieldInfo;
import javassist.bytecode.annotation.Annotation;
import javassist.bytecode.annotation.ArrayMemberValue;
import javassist.bytecode.annotation.EnumMemberValue;
import javassist.bytecode.annotation.StringMemberValue;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.net.URL;
import java.net.URLClassLoader;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Enumeration;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;
import java.util.stream.Collectors;

public class JarLoader {
    public static void main(String[] args) {

        String dependencyPath = "C:\\Users\\gabid\\Desktop\\cpm\\lib\\DynamX-4.1.0-dev19-all-deobf.jar";
        String jarPath = "aa.jar";

    }

    public static AddonProperties loadJar(String dependencyPath, String jarPath) {
        try {
            URLClassLoader urlClassLoader = createURLClassLoader(dependencyPath);
            Thread.currentThread().setContextClassLoader(urlClassLoader);
            return readAnnotations(jarPath);
        } catch (IOException | ClassNotFoundException e) {
            AddonProperties addonProperties = new AddonProperties();
            addonProperties.state = AddonConverterState.ERRORED;
            return addonProperties;
        }
    }

    private static URLClassLoader createURLClassLoader(String jarPath) throws IOException {
        File file = new File(jarPath);
        URL url = file.toURI().toURL();
        return new URLClassLoader(new URL[]{url}, JarLoader.class.getClassLoader());
    }

    private static void loadExternalJar(String jarPath) throws IOException {
        File file = new File(jarPath);
        URL url = file.toURI().toURL();
        URLClassLoader urlClassLoader = (URLClassLoader) ClassLoader.getSystemClassLoader();
        Class<?> urlClassLoaderClass = URLClassLoader.class;

        try {
            java.lang.reflect.Method method = urlClassLoaderClass.getDeclaredMethod("addURL", URL.class);
            method.setAccessible(true);
            method.invoke(urlClassLoader, url);
        } catch (Exception e) {
            throw new IOException("Failed to load external JAR", e);
        }
    }

    public static AddonProperties readAnnotations(String jarPath) throws IOException, ClassNotFoundException {
        AddonProperties addonProperties = new AddonProperties();
        addonProperties.name = "test";
        JarFile jarFile = new JarFile(jarPath);
        Enumeration<JarEntry> entries = jarFile.entries();
        ClassPool classPool = ClassPool.getDefault();

        while (entries.hasMoreElements()) {
            JarEntry entry = entries.nextElement();
            String entryName = entry.getName();

            if (entryName.endsWith(".class")) {
                try (InputStream inputStream = jarFile.getInputStream(entry)) {
                    CtClass ctClass = classPool.makeClass(inputStream);

                    ClassFile classFile = ctClass.getClassFile();
                    // get all annotations on the class
                    AnnotationsAttribute visibleClass = (AnnotationsAttribute) classFile.getAttribute(AnnotationsAttribute.visibleTag);
                    if (visibleClass != null) {
                        for (Annotation annotation : visibleClass.getAnnotations()) {
                            if (annotation.getTypeName().equals("fr.dynamx.api.contentpack.DynamXAddon")) {
                                addonProperties.name = ((StringMemberValue) annotation.getMemberValue("name")).getValue();
                                addonProperties.version = ((StringMemberValue) annotation.getMemberValue("version")).getValue();
                            }
                            if (annotation.getTypeName().equals("fr.dynamx.api.contentpack.registry.RegisteredSubInfoType")) {
                                List<String> propertiesList = new ArrayList<>();
                                for (FieldInfo field : classFile.getFields()) {
                                    AnnotationsAttribute visibleField = (AnnotationsAttribute) field.getAttribute(AnnotationsAttribute.visibleTag);
                                    if (visibleField != null) {
                                        for (Annotation fieldAnnotation : visibleField.getAnnotations()) {
                                            if (fieldAnnotation.getTypeName().equals("fr.dynamx.api.contentpack.registry.PackFileProperty")) {
                                                propertiesList.addAll(Arrays.asList(((ArrayMemberValue) fieldAnnotation.getMemberValue("configNames")).getValue()).stream().map(memberValue -> ((StringMemberValue) memberValue).getValue()).collect(Collectors.toList()));
                                            }
                                        }
                                    }
                                }
                                ArrayMemberValue registries = (ArrayMemberValue) annotation.getMemberValue("registries");
                                // its an array of enum, get only the name of the enum
                                List<String> enums = Arrays.stream(registries.getValue()).map(memberValue -> ((EnumMemberValue) memberValue).getValue()).collect(Collectors.toList());
                                for (String anEnum : enums) {
                                    switch (anEnum) {
                                        case "WHEELED_VEHICLES": {
                                            addonProperties.vehiclesProperties.put(((StringMemberValue) annotation.getMemberValue("name")).getValue(), propertiesList);
                                            break;
                                        }
                                        case "TRAILERS": {
                                            addonProperties.trailerProperties.put(((StringMemberValue) annotation.getMemberValue("name")).getValue(), propertiesList);
                                            break;
                                        }
                                        case "BOATS": {
                                            addonProperties.boatProperties.put(((StringMemberValue) annotation.getMemberValue("name")).getValue(), propertiesList);
                                            break;
                                        }
                                        case "HELICOPTER": {
                                            addonProperties.helicopterProperties.put(((StringMemberValue) annotation.getMemberValue("name")).getValue(), propertiesList);
                                            break;
                                        }
                                        case "ITEMS": {
                                            addonProperties.itemProperties.put(((StringMemberValue) annotation.getMemberValue("name")).getValue(), propertiesList);
                                            break;
                                        }
                                        case "ARMORS": {
                                            addonProperties.armorProperties.put(((StringMemberValue) annotation.getMemberValue("name")).getValue(), propertiesList);
                                            break;
                                        }
                                        case "BLOCKS": {
                                            addonProperties.blockProperties.put(((StringMemberValue) annotation.getMemberValue("name")).getValue(), propertiesList);
                                            break;
                                        }
                                        case "PROPS": {
                                            addonProperties.propProperties.put(((StringMemberValue) annotation.getMemberValue("name")).getValue(), propertiesList);
                                            break;
                                        }
                                        case "WHEELS": {
                                            addonProperties.wheelProperties.put(((StringMemberValue) annotation.getMemberValue("name")).getValue(), propertiesList);
                                            break;
                                        }
                                        case "CAR_ENGINES": {
                                            addonProperties.carEnginesProperties.put(((StringMemberValue) annotation.getMemberValue("name")).getValue(), propertiesList);
                                            break;
                                        }
                                        case "HELICOPTER_ENGINES": {
                                            addonProperties.helicopterEngineProperties.put(((StringMemberValue) annotation.getMemberValue("name")).getValue(), propertiesList);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }

                } catch (Exception e) {
                    e.printStackTrace();
                }
            }
        }

        jarFile.close();
        if (addonProperties.name == null || addonProperties.version == null) {
            addonProperties.state = AddonConverterState.NOTDYNAMXADDON;
            return addonProperties;
        }
        addonProperties.state = AddonConverterState.DONE;
        return addonProperties;
    }
}
