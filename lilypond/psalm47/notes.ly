melody = \relative c'' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  r8 a2 fis2 d4 e4 fis4 g2 fis4 e4 e4 d2 r2 a'2 a2 b4 c4 d4 a2 b4 g4 a4 f2 r2 \break

  % Line 2
  d2 e2 f4 g4 f4 e2 d4 d4 c4 d2 r2 \break

  % Line 3
  a2 b2 a4 g4 f4 f2 e4 f4 g4 a2 r2 \break

  % Line 4
  d2 c2 b4 b4 a4 b2 a4 g4 f4 e2 r2 \break

  % Line 5
  d2 f2 a4 g4 f4 e2 d4 d4 c4 d1 \bar "|."
}
