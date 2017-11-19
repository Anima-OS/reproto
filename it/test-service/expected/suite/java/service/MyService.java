package service;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import common._1_0_0.Entry;
import io.grpc.BindableService;
import io.grpc.CallOptions;
import io.grpc.Channel;
import io.grpc.MethodDescriptor;
import io.grpc.MethodDescriptor.Marshaller;
import io.grpc.ServerServiceDefinition;
import io.grpc.stub.AbstractStub;
import io.grpc.stub.ClientCalls;
import io.grpc.stub.ServerCalls;
import io.grpc.stub.StreamObserver;
import io.reproto.MapperProvider;
import java.io.ByteArrayInputStream;
import java.io.InputStream;
import javax.annotation.Generated;

public interface MyService {
  public static final MethodDescriptor<Void, Void> METHOD_UNKNOWN = 
    MethodDescriptor.<Void, Void>newBuilder()
      .setType(MethodDescriptor.MethodType.UNKNOWN)
      .setFullMethodName(MethodDescriptor.generateFullMethodName("service.MyService", "unknown"))
      .setRequestMarshaller(new VoidMarshaller())
      .setResponseMarshaller(new VoidMarshaller())
      .build();

  public static final MethodDescriptor<Void, Entry> METHOD_UNKNOWN_RETURN = 
    MethodDescriptor.<Void, Entry>newBuilder()
      .setType(MethodDescriptor.MethodType.UNKNOWN)
      .setFullMethodName(MethodDescriptor.generateFullMethodName("service.MyService", "unknown_return"))
      .setRequestMarshaller(new VoidMarshaller())
      .setResponseMarshaller(new JsonMarshaller(new TypeReference<Entry>(){}))
      .build();

  public static final MethodDescriptor<Entry, Void> METHOD_UNKNOWN_ARGUMENT = 
    MethodDescriptor.<Entry, Void>newBuilder()
      .setType(MethodDescriptor.MethodType.UNKNOWN)
      .setFullMethodName(MethodDescriptor.generateFullMethodName("service.MyService", "unknown_argument"))
      .setRequestMarshaller(new JsonMarshaller(new TypeReference<Entry>(){}))
      .setResponseMarshaller(new VoidMarshaller())
      .build();

  public static final MethodDescriptor<Entry, Entry> METHOD_UNARY = 
    MethodDescriptor.<Entry, Entry>newBuilder()
      .setType(MethodDescriptor.MethodType.UNARY)
      .setFullMethodName(MethodDescriptor.generateFullMethodName("service.MyService", "unary"))
      .setRequestMarshaller(new JsonMarshaller(new TypeReference<Entry>(){}))
      .setResponseMarshaller(new JsonMarshaller(new TypeReference<Entry>(){}))
      .build();

  public static final MethodDescriptor<Entry, Entry> METHOD_SERVER_STREAMING = 
    MethodDescriptor.<Entry, Entry>newBuilder()
      .setType(MethodDescriptor.MethodType.SERVER_STREAMING)
      .setFullMethodName(MethodDescriptor.generateFullMethodName("service.MyService", "server_streaming"))
      .setRequestMarshaller(new JsonMarshaller(new TypeReference<Entry>(){}))
      .setResponseMarshaller(new JsonMarshaller(new TypeReference<Entry>(){}))
      .build();

  public static final MethodDescriptor<Entry, Entry> METHOD_CLIENT_STREAMING = 
    MethodDescriptor.<Entry, Entry>newBuilder()
      .setType(MethodDescriptor.MethodType.CLIENT_STREAMING)
      .setFullMethodName(MethodDescriptor.generateFullMethodName("service.MyService", "client_streaming"))
      .setRequestMarshaller(new JsonMarshaller(new TypeReference<Entry>(){}))
      .setResponseMarshaller(new JsonMarshaller(new TypeReference<Entry>(){}))
      .build();

  public static final MethodDescriptor<Entry, Entry> METHOD_BIDI_STREAMING = 
    MethodDescriptor.<Entry, Entry>newBuilder()
      .setType(MethodDescriptor.MethodType.BIDI_STREAMING)
      .setFullMethodName(MethodDescriptor.generateFullMethodName("service.MyService", "bidi_streaming"))
      .setRequestMarshaller(new JsonMarshaller(new TypeReference<Entry>(){}))
      .setResponseMarshaller(new JsonMarshaller(new TypeReference<Entry>(){}))
      .build();

  @Generated("Generated by ReProto")
  static class ClientStub extends AbstractStub<ClientStub> {
    public ClientStub(
      final Channel channel
    ) {
      super(channel);
    }

    public ClientStub(
      final Channel channel,
      final CallOptions callOptions
    ) {
      super(channel, callOptions);
    }

    @Override
    protected ClientStub build(final Channel channel, final CallOptions callOptions) {
      return new ClientStub(channel, callOptions);
    }

    /**
     * <pre>
     * UNKNOWN
     * </pre>
     */
    public StreamObserver<Void> unknown(final StreamObserver<Void> observer) {
      return ClientCalls.asyncBidiStreamingCall(getChannel().newCall(METHOD_UNKNOWN, getCallOptions()), observer);
    }

    /**
     * <pre>
     * UNKNOWN
     * </pre>
     */
    public StreamObserver<Void> unknownReturn(final StreamObserver<Entry> observer) {
      return ClientCalls.asyncBidiStreamingCall(getChannel().newCall(METHOD_UNKNOWN_RETURN, getCallOptions()), observer);
    }

    /**
     * <pre>
     * UNKNOWN
     * </pre>
     */
    public StreamObserver<Entry> unknownArgument(final StreamObserver<Void> observer) {
      return ClientCalls.asyncBidiStreamingCall(getChannel().newCall(METHOD_UNKNOWN_ARGUMENT, getCallOptions()), observer);
    }

    /**
     * <pre>
     * UNARY
     * </pre>
     */
    public void unary(final Entry request, final StreamObserver<Entry> observer) {
      ClientCalls.asyncUnaryCall(getChannel().newCall(METHOD_UNARY, getCallOptions()), request, observer);
    }

    /**
     * <pre>
     * SERVER_STREMAING
     * </pre>
     */
    public void serverStreaming(final Entry request, final StreamObserver<Entry> observer) {
      ClientCalls.asyncServerStreamingCall(getChannel().newCall(METHOD_SERVER_STREAMING, getCallOptions()), request, observer);
    }

    /**
     * <pre>
     * CLIENT_STREAMING
     * </pre>
     */
    public StreamObserver<Entry> clientStreaming(final StreamObserver<Entry> observer) {
      return ClientCalls.asyncClientStreamingCall(getChannel().newCall(METHOD_CLIENT_STREAMING, getCallOptions()), observer);
    }

    /**
     * <pre>
     * BIDI_STREAMING
     * </pre>
     */
    public StreamObserver<Entry> bidiStreaming(final StreamObserver<Entry> observer) {
      return ClientCalls.asyncBidiStreamingCall(getChannel().newCall(METHOD_BIDI_STREAMING, getCallOptions()), observer);
    }
  }

  @Generated("Generated by ReProto")
  abstract static class ServerStub implements BindableService {
    /**
     * <pre>
     * UNKNOWN
     * </pre>
     */
    public StreamObserver<Void> unknown(final StreamObserver<Void> observer) {
      return ServerCalls.asyncUnimplementedStreamingCall(METHOD_UNKNOWN, observer);
    }

    /**
     * <pre>
     * UNKNOWN
     * </pre>
     */
    public StreamObserver<Void> unknownReturn(final StreamObserver<Entry> observer) {
      return ServerCalls.asyncUnimplementedStreamingCall(METHOD_UNKNOWN_RETURN, observer);
    }

    /**
     * <pre>
     * UNKNOWN
     * </pre>
     */
    public StreamObserver<Entry> unknownArgument(final StreamObserver<Void> observer) {
      return ServerCalls.asyncUnimplementedStreamingCall(METHOD_UNKNOWN_ARGUMENT, observer);
    }

    /**
     * <pre>
     * UNARY
     * </pre>
     */
    public void unary(final Entry request, final StreamObserver<Entry> observer) {
      ServerCalls.asyncUnimplementedUnaryCall(METHOD_UNARY, observer);
    }

    /**
     * <pre>
     * SERVER_STREMAING
     * </pre>
     */
    public void serverStreaming(final Entry request, final StreamObserver<Entry> observer) {
      ServerCalls.asyncUnimplementedUnaryCall(METHOD_SERVER_STREAMING, observer);
    }

    /**
     * <pre>
     * CLIENT_STREAMING
     * </pre>
     */
    public StreamObserver<Entry> clientStreaming(final StreamObserver<Entry> observer) {
      return ServerCalls.asyncUnimplementedStreamingCall(METHOD_CLIENT_STREAMING, observer);
    }

    /**
     * <pre>
     * BIDI_STREAMING
     * </pre>
     */
    public StreamObserver<Entry> bidiStreaming(final StreamObserver<Entry> observer) {
      return ServerCalls.asyncUnimplementedStreamingCall(METHOD_BIDI_STREAMING, observer);
    }

    @Override
    public ServerServiceDefinition bindService() {
      return ServerServiceDefinition
        .builder("service.MyService")
        .addMethod(METHOD_UNKNOWN, ServerCalls.asyncBidiStreamingCall(this::unknown))
        .addMethod(METHOD_UNKNOWN_RETURN, ServerCalls.asyncBidiStreamingCall(this::unknownReturn))
        .addMethod(METHOD_UNKNOWN_ARGUMENT, ServerCalls.asyncBidiStreamingCall(this::unknownArgument))
        .addMethod(METHOD_UNARY, ServerCalls.asyncUnaryCall(this::unary))
        .addMethod(METHOD_SERVER_STREAMING, ServerCalls.asyncServerStreamingCall(this::serverStreaming))
        .addMethod(METHOD_CLIENT_STREAMING, ServerCalls.asyncClientStreamingCall(this::clientStreaming))
        .addMethod(METHOD_BIDI_STREAMING, ServerCalls.asyncBidiStreamingCall(this::bidiStreaming))
        .build();
    }
  }

  public static class JsonMarshaller<T> implements MethodDescriptor.Marshaller<T> {
    private final ObjectMapper mapper;
    private final TypeReference<T> type;

    public JsonMarshaller(
      final TypeReference<T> type
    ) {
      this.mapper = MapperProvider.get();
      this.type = type;
    }

    @Override
    public T parse(final InputStream stream) {
      try {
        return this.mapper.readValue(stream, this.type);
      } catch (final Exception e) {
        throw new RuntimeException(e);
      }
    }

    @Override
    public InputStream stream(final T value) {
      final byte[] bytes;
      try {
        bytes = this.mapper.writeValueAsBytes(value);
      } catch (final Exception e) {
        throw new RuntimeException(e);
      }
      return new ByteArrayInputStream(bytes);
    }
  }

  public static class VoidMarshaller implements MethodDescriptor.Marshaller<Void> {
    @Override
    public Void parse(final InputStream stream) {
      return null;
    }

    @Override
    public InputStream stream(final Void value) {
      return new ByteArrayInputStream(new byte[0]);
    }
  }
}
