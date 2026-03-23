melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 g'2 e2 a4 b4 c4 c4 b2 \break

  % Line 2
  a2 c2 g4 e4 f4 e4 d2 c2 r2 \break

  % Line 3
  g'2 e2 a4 b4 c4 c4 b2 \break

  % Line 4
  a2 c2 g4 e4 f4 e4 d2 c2 r2 \break

  % Line 5
  e2 fis2 g4 a4 a4 gis4 a2 \break

  % Line 6
  b4 c4 d4 e4 d4 d4 c1 \bar "|."
}
